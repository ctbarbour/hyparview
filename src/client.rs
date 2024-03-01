use crate::message::ProtocolMessage;
use futures::SinkExt;
use std::net::{IpAddr, SocketAddr};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub struct Client {
    connection: Framed<TcpStream, LengthDelimitedCodec>,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client, std::io::Error> {
        let socket = TcpStream::connect(addr).await?;

        Ok(Client {
            connection: Framed::new(socket, LengthDelimitedCodec::new()),
        })
    }

    pub async fn join(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}
