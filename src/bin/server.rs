use hyparview::server;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Debug)]
struct Config {
    local_endpoint: SocketAddr,
    active_random_walk_length: u32,
    passive_random_walk_length: u32,
    active_view_capacity: u32,
    passive_view_capacity: u32,
    shuffle_ttl: u32,
    shuffle_active_view_count: u32,
    shuffle_passive_view_count: u32,
    shuffle_interval: u32,
}

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
