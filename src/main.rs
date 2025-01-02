use tonic::transport::Endpoint;
use tower::service_fn;
use std::net::SocketAddr;
use tokio::main;
use udpconnector::UdpConnector; 
use pingpong::ping_pong_client::PingPongClient;
use pingpong::PingRequest;
use tower::Service;
mod udpconnector;

pub mod pingpong {
    tonic::include_proto!("pingpong");
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define local and remote addresses for UDP.
    let local_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let remote_addr: SocketAddr = "127.0.0.1:54321".parse().unwrap();

    // Initialize the UDP connector.
    let udp_connector = UdpConnector::new(local_addr, remote_addr);

    // Use the UDP connector to create a tonic channel.
    let channel = Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(service_fn(move |uri| {
            let mut connector = udp_connector.clone();
            async move { connector.call(uri).await }
        }))
        .await?;

    // Initialize the gRPC client.
    let mut client = PingPongClient::new(channel);

    // Create and send a Ping request.
    let request = tonic::Request::new(PingRequest {
        message: "Ping!".into(),
    });

    // Await the response.
    let response = client.send_ping(request).await?;

    // Print the response.
    println!("Response received: {}", response.into_inner().message);

    Ok(())
}
