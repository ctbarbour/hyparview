use hyparview::server;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("hyparview=info".parse()?))
        .with_span_events(FmtSpan::FULL)
        .init();
    // Parse the address we're going to run this server on
    // and set up our TCP listener to accept connections.
    let addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("127.0.0.1:8080"))
        .parse()
        .unwrap();

    let listener = TcpListener::bind(addr).await?;

    tracing::info!("server running on {}", addr);
    server::run(listener).await;

    Ok(())
}
