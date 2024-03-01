use crate::PeerState;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neighbor {
    pub sender: SocketAddr,
    pub high_priority: bool,
}

impl Neighbor {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_neighbor(self).await;
        Ok(())
    }
}
