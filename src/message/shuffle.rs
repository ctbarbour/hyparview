use crate::PeerState;
use std::collections::HashSet;
use std::net::SocketAddr;

pub struct ShuffleMessage {
    pub sender: SocketAddr,
    pub origin: SocketAddr,
    pub nodes: HashSet<SocketAddr>,
    pub ttl: u32,
}

impl ShuffleMessage {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_shuffle(self).await;
        Ok(())
    }
}
