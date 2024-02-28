use crate::PeerState;
use std::net::SocketAddr;

pub struct NeighborMessage {
    pub sender: SocketAddr,
    pub high_priority: bool,
}

impl NeighborMessage {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_neighbor(self).await;
        Ok(())
    }
}
