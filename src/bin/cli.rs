use futures::SinkExt;
use hyparview::client::Client;
use hyparview::codec::HyParViewCodec;
use hyparview::messages::{JoinMessage, ShuffleMessage};
use std::collections::HashSet;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the address we're going to run this server on
    // and set up our TCP listener to accept connections.
    let addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("127.0.0.1:8080"))
        .parse()
        .unwrap();

    let mut client = Client::connect(addr).await?;

    client.join();

    Ok(())
}
