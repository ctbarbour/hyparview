use crate::message::ProtocolMessage;
use crate::Connection;
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<Client> {
        let socket = TcpStream::connect(addr).await?;

        Ok(Client {
            connection: Connection::new(socket),
        })
    }

    pub async fn join(&mut self) -> crate::Result<()> {
        let join_message = ProtocolMessage::join("127.0.0.1:8088".parse().unwrap());
        self.send(&join_message).await
    }

    pub async fn send(&mut self, message: &ProtocolMessage) -> crate::Result<()> {
        self.connection.write_frame(message).await?;
        Ok(())
    }
}
