use tokio_util::codec::Framed;
use tokio::net::TcpStream;
use std::net::SocketAddr;
use crate::codec::HyParViewCodec;

pub struct Connection {
    transport: Framed<TcpStream, HyParViewCodec>,
    peer: SocketAddr,
}

impl Connection {
    pub fn new(peer: SocketAddr, stream: TcpStream) -> Connection {
        Connection {
            transport: Framed::new(stream, HyParViewCodec::new()),
            peer: peer
        }
    }

    pub fn peer() -> SocketAddr {
        &self.peer
    }
}
