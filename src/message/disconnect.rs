use crate::PeerState;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct DisconnectMessage {
    pub sender: SocketAddr,
    pub alive: bool,
    pub respond: bool,
}

impl DisconnectMessage {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_disconnect(self).await;
        Ok(())
    }
}
