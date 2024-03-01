use crate::PeerState;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shuffle {
    pub sender: SocketAddr,
    pub origin: SocketAddr,
    pub nodes: Vec<SocketAddr>,
    pub ttl: u32,
}

impl Shuffle {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_shuffle(self).await;
        Ok(())
    }
}
