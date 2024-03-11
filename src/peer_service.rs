use crate::Connection;
use crate::PeerState;
use crate::ProtocolMessage;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, Notify};

type Rx = mpsc::UnboundedReceiver<ProtocolMessage>;
type Tx = mpsc::UnboundedSender<ProtocolMessage>;

#[derive(Debug)]
pub struct PeerService {
    // This is maybe better to call the NodeID since it's not actually
    // the peer_addr you'd get when accepting a connection from a listener
    peer_addr: SocketAddr,
    connection: Connection,
    tx: Tx,
    rx: Rx,
    shutdown: Arc<Notify>,
}

impl PeerService {
    pub(crate) fn new(peer_addr: SocketAddr, stream: TcpStream) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let connection = Connection::new(stream),
        let shutdown = Arc::new(Notify::new());
        Self { peer_addr, connection, tx, rx, shutdown }
    }

    pub(crate) fn send(&self, message: ProtocolMessage) -> crate::Result<()> {
        Ok(self.tx.send(message)?)
    }

    pub(crate) async fn handle_connection(
        &mut self,
        peer_state: PeerState,
    ) -> crate::Result<()> {
        loop {
            tokio::select!(
                _ = self.shutdown.notified() => { break }

                Some(message) = self.rx.recv() => {
                    // This will return on error instead of breaking from the loop
                    self.connection.write_frame(&message).await?;
                }

                res = self.connection.read_frame() => match res {
                    // This will return on error instead of breaking from the loop
                    Ok(Some(message)) => message.apply(&peer_state).await?,
                    Ok(None) => break,
                    Err(_) => break,
                }
            )
        }

        peer_state.drop_active_connection(self.peer_addr);
        let disconnect_message = ProtocolMessage::disconnect(
            self.peer_addr,
            false,
            false,
        );

        peer_state.broadcast(disconnect_message);

        Ok(())
    }
}
