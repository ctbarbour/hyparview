use crate::PeerState;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Join {
    pub sender: SocketAddr,
}

impl Join {
    pub(crate) async fn apply(&self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_join(self.clone()).await;
        Ok(())
    }
}
