use crate::PeerState;
use std::collections::HashSet;
use std::net::SocketAddr;

pub struct ShuffleReplyMessage {
    pub sender: SocketAddr,
    pub nodes: HashSet<SocketAddr>,
}

impl ShuffleReplyMessage {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_shuffle_reply(self).await;
        Ok(())
    }
}
