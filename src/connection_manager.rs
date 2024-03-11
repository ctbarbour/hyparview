use crate::Connection;
use crate::PeerState;
use crate::ProtocolMessage;
use crate::PeerStateDropGuard;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex, Notify};

type Rx = mpsc::UnboundedReceiver<ProtocolMessage>;
type Tx = mpsc::UnboundedSender<ProtocolMessage>;

#[derive(Debug)]
struct PeerConnection {
    // This is maybe better to call the NodeID since it's not actually
    // the peer_addr you'd get when accepting a connection from a listener
    peer_addr: SocketAddr,
    connection: Connection,
    tx: Tx,
    rx: Rx,
    shutdown: Arc<Notify>,
}

impl PeerConnection {
    pub(crate) fn new(peer_addr: SocketAddr, stream: TcpStream) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let shutdown = Arc::new(Notify::new());
        let connection = Connection::new(stream);
        Self { peer_addr, connection, tx, rx, shutdown }
    }

    pub(crate) async fn handle_connection(
        &mut self,
        peer_state: PeerState,
    ) -> crate::Result<()> {
        tokio::spawn(async move {
            loop {
                tokio::select!(
                    _ = self.shutdown.notified() => { break; }

                    Some(message) = self.rx.recv() => {
                        self.connection.write_frame(&message).await;
                    }

                    res = self.connection.read_frame() => match res {
                        Ok(Some(message)) => message.apply(&mut peer_state).await?,
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
        });

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionManager {
    active_connections: Arc<Mutex<HashMap<SocketAddr, PeerConnection>>>,
}

impl ConnectionManager {
    pub(crate) async fn send(
        &mut self,
        peer: SocketAddr,
        message: &ProtocolMessage,
    ) -> crate::Result<()> {
        let active_connections = self.active_connections.lock().await;
        if let Some(signals) = active_connections.get(&peer) {
            &signals.tx.send(message.clone());
        }

        Ok(())
    }

    pub(crate) async fn broadcast(&mut self, message: &ProtocolMessage) -> crate::Result<()> {
        let active_connections = self.active_connections.lock().await;

        for (_, signals) in active_connections.iter() {
            signals.tx.send(message.clone());
        }

        Ok(())
    }
}
