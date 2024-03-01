use crate::PeerState;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShuffleReply {
    pub sender: SocketAddr,
    pub nodes: Vec<SocketAddr>,
}

impl ShuffleReply {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_shuffle_reply(self).await;
        Ok(())
    }
}
