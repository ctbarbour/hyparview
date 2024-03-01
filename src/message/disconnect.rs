use crate::PeerState;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disconnect {
    pub sender: SocketAddr,
    pub alive: bool,
    pub respond: bool,
}

impl Disconnect {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_disconnect(self).await;
        Ok(())
    }
}
