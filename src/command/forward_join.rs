use std::net::SocketAddr;
use crate::state::PeerState;

pub struct ForwardJoin {
    peer: SocketAddr,
    ttl: u32
}

impl ForwardJoin {
    pub(crate) fn new(peer: SocketAddr, ttl: u32) -> Self {
        assert!(ttl >= 0);

        ForwardJoin {
            peer: peer,
            ttl: ttl
        }
    }

    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_forward_join(self).await?;
        Ok(())
    }
}
