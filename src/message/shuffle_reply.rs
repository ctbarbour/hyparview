use crate::PeerState;
use std::net::SocketAddr;
use std::collections::HashSet;

pub struct ShuffleReplyMessage {
    pub sender: SocketAddr,
    pub nodes: HashSet<SocketAddr>
}

impl ShuffleReplyMessage {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_shuffle_reply(self).await;
        Ok(())
    }
}
