use crate::PeerState;
use std::net::SocketAddr;

pub struct ForwardJoinMessage {
    pub sender: SocketAddr,
    pub peer: SocketAddr,
    pub ttl: u32,
}

impl ForwardJoinMessage {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_forward_join(self).await;
        Ok(())
    }
}
