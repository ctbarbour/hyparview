use crate::codec::HyParViewCodec;
use crate::messages::*;
use futures::SinkExt;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::codec::Framed;

pub struct Client {
    connection: Framed<TcpStream, HyParViewCodec>,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client, std::io::Error> {
        let socket = TcpStream::connect(addr).await?;

        Ok(Client {
            connection: Framed::new(socket, HyParViewCodec::new()),
        })
    }

    pub async fn join(&mut self) -> Result<(), std::io::Error> {
        self.connection.send(Box::new(JoinMessage {})).await?;
        Ok(())
    }
}
