use crate::Connection;
use crate::PeerState;
use crate::ProtocolMessage;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex, Notify};

type Rx = mpsc::UnboundedReceiver<ProtocolMessage>;
type Tx = mpsc::UnboundedSender<ProtocolMessage>;

#[derive(Debug, Clone)]
struct ConnectionSignals {
    tx: Tx,
    shutdown: Arc<Notify>,
}

#[derive(Debug, Clone)]
struct ActiveConnections {
    connections: Arc<Mutex<HashMap<SocketAddr, ConnectionSignals>>>,
}

#[derive(Debug, Clone)]
pub struct ConnectionManager {
    state: PeerState,
    active_connections: ActiveConnections,
}

impl ConnectionManager {
    fn state(&self) -> PeerState {
        self.state.clone()
    }

    pub(crate) async fn send(
        &mut self,
        peer: SocketAddr,
        message: &ProtocolMessage,
    ) -> crate::Result<()> {
        let active_connections = self.active_connections.connections.lock().await;
        if let Some(signals) = active_connections.get(&peer) {
            &signals.tx.send(message.clone());
        }

        Ok(())
    }

    pub(crate) async fn broadcast(&mut self, message: &ProtocolMessage) -> crate::Result<()> {
        let active_connections = self.active_connections.connections.lock().await;

        for (_, signals) in active_connections.iter() {
            signals.tx.send(message.clone());
        }

        Ok(())
    }

    pub(crate) async fn handle_connection(
        &mut self,
        peer_addr: SocketAddr,
        mut connection: Connection,
    ) -> crate::Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let connection_signals = ConnectionSignals {
            tx: tx,
            shutdown: Arc::new(Notify::new()),
        };

        let mut task_state = self.state();
        let shutdown = connection_signals.shutdown.clone();
        {
            let mut active_connections = self.active_connections.connections.lock().await;
            active_connections.insert(peer_addr, connection_signals);
        }

        let mut active_connections = self.active_connections.clone();

        tokio::spawn(async move {
            loop {
                tokio::select!(
                    _ = shutdown.notified() => { break; }

                    Some(message) = rx.recv() => {
                        connection.write_frame(&message).await;
                    }

                    res = connection.read_frame() => match res {
                        Ok(Some(message)) => message.apply(&mut task_state).await?,
                        Ok(None) => break,
                        Err(err) => return Err(err),
                    }
                )
            }

            {
                let mut active_connections = active_connections.connections.lock().await;
                active_connections.remove(&peer_addr);
                let disconnect_message = ProtocolMessage::disconnect(
                    task_state.local_peer().await.unwrap(),
                    false,
                    false,
                );
                for (_, signals) in active_connections.iter() {
                    signals.tx.send(disconnect_message.clone());
                }
            }

            Ok(())
        });
        Ok(())
    }
}
