use tokio::net::{UdpSocket, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // UDP server configuration
    let udp_local_addr: SocketAddr = "127.0.0.1:54321".parse().unwrap();
    
    // Bind to the UDP socket
    let udp_socket = Arc::new(UdpSocket::bind(udp_local_addr).await?);
    println!("Listening for UDP packets on {}", udp_local_addr);

    // Map to store TCP connections keyed by source port
    let tcp_connections: Arc<RwLock<HashMap<u16, Arc<Mutex<TcpStream>>>>> = Arc::new(RwLock::new(HashMap::new()));

    loop {
        let udp_socket = Arc::clone(&udp_socket);
        let tcp_connections = Arc::clone(&tcp_connections);
        let mut buffer = vec![0u8; 1024];

        // Receive UDP packet
        let (len, addr) = udp_socket.recv_from(&mut buffer).await?;
        let source_port = addr.port();
        println!("Received {} bytes from {}", len, addr);

        tokio::spawn(async move {
            let tcp_stream = {
                let mut connections = tcp_connections.write().await;
                if let Some(existing_stream) = connections.get(&source_port) {
                    Arc::clone(existing_stream)
                } else {
                    // Create a new TCP connection if one does not exist for this source port
                    let tcp_server_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
                    let new_stream = TcpStream::connect(tcp_server_addr).await.unwrap();
                    let new_stream = Arc::new(Mutex::new(new_stream));
                    connections.insert(source_port, Arc::clone(&new_stream));
                    new_stream
                }
            };

            let mut tcp_stream = tcp_stream.lock().await;

            // Send data to the TCP server
            if let Err(e) = tcp_stream.write_all(&buffer[..len]).await {
                eprintln!("Failed to send data to TCP server: {}", e);
                return;
            }
            println!("Successfully forwarded data to TCP server");

            // Receive response from the TCP server
            let mut response = vec![0u8; 1024];
            match tcp_stream.read(&mut response).await {
                Ok(response_len) => {
                    // Send the response back to the UDP client
                    if let Err(e) = udp_socket.send_to(&response[..response_len], addr).await {
                        eprintln!("Failed to send response to UDP client: {}", e);
                    } else {
                        println!("Successfully sent response to UDP client");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to receive response from TCP server: {}", e);
                }
            }
        });
    }
}
