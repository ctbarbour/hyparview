use crate::ProtocolMessage;
use crate::Connection;
use crate::PeerState;
use std::collections::HashMap;
use tokio::sync::{mpsc, Mutex};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use std::sync::Arc;

type Rx = mpsc::UnboundedReceiver<ProtocolMessage>;
type Tx = mpsc::UnboundedSender<ProtocolMessage>;

pub struct ConnectionManager {
    state: PeerState,
    active_connections: Mutex<HashMap<SocketAddr, Tx>>,
}

impl ConnectionManager {
    fn state(&self) -> PeerState {
        self.state.clone()
    }

    pub(crate) async fn handle_connection(&mut self, mut connection: Connection) -> crate::Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        {
            let mut guard = self.active_connections.lock().await;
            guard.insert(connection.peer_addr(), tx);
        }

        let mut task_state = self.state();
        tokio::spawn(async move {
            loop {
                tokio::select!(
                    Some(message) = rx.recv() => {
                        connection.write_frame(&message).await?;
                    }

                    res = connection.read_frame() => match res {
                        Ok(Some(message)) => message.apply(&mut task_state).await?,
                        Ok(None) => return Ok(()),
                        Err(err) => return Err(err),
                    }
                )
            }
        });
        Ok(())
    }
}
