use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::UdpSocket;
use tonic::transport::Uri;
use tower::Service;
use std::{pin::Pin, task::{Context, Poll}, net::SocketAddr};
use futures::future::BoxFuture;

#[derive(Clone)]
pub struct UdpConnector {
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
}

impl UdpConnector {
    pub fn new(local_addr: SocketAddr, remote_addr: SocketAddr) -> Self {
        Self {
            local_addr,
            remote_addr,
        }
    }
}

impl Service<Uri> for UdpConnector {
    type Response = UdpStream;
    type Error = std::io::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _uri: Uri) -> Self::Future {
        let local_addr = self.local_addr;
        let remote_addr = self.remote_addr;

        Box::pin(async move {
            let socket = UdpSocket::bind(local_addr).await?;
            socket.connect(remote_addr).await?;
            Ok(UdpStream::new(socket))
        })
    }
}

pub struct UdpStream {
    socket: UdpSocket,
}

impl UdpStream {
    pub fn new(socket: UdpSocket) -> Self {
        Self { socket }
    }
}

impl AsyncRead for UdpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let mut temp_buf = vec![0u8; 1024];
        let mut read_buf = tokio::io::ReadBuf::new(&mut temp_buf);
        futures::ready!(Pin::new(&mut self.socket).poll_recv(cx, &mut read_buf))?;
        let n = read_buf.filled().len();
        buf.put_slice(&temp_buf[..n]);
        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for UdpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.socket).poll_send(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}
