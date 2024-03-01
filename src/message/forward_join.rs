use crate::PeerState;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardJoin {
    pub sender: SocketAddr,
    pub peer: SocketAddr,
    pub ttl: u32,
}

impl ForwardJoin {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_forward_join(self).await;
        Ok(())
    }
}
