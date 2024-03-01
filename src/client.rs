use crate::message::ProtocolMessage;
use crate::Connection;
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client, std::io::Error> {
        let socket = TcpStream::connect(addr).await?;

        Ok(Client {
            connection: Connection::new(socket),
        })
    }

    pub async fn join(&mut self) -> Result<(), std::io::Error> {
        let join_message = ProtocolMessage::join("127.0.0.1:8080".parse().unwrap());
        self.connection.write_frame(join_message).await?;
        Ok(())
    }
}
