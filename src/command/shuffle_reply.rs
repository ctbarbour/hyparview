use crate::state::PeerState;
use std::net::SocketAddr;
use std::collections::HashSet;

pub struct ShuffleReply {
    nodes: HashSet<SocketAddr>
}

impl ShuffleReply {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_shuffle_reply(self).await?;
        Ok(())
    }
}
