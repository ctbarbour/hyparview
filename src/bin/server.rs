use hyparview::server;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the address we're going to run this server on
    // and set up our TCP listener to accept connections.
    let addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("127.0.0.1:8080"))
        .parse()
        .unwrap();

    let listener = TcpListener::bind(addr).await?;

    server::run(listener).await;

    Ok(())
}
