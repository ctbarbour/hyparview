use crate::state::PeerState;
use std::collections::HashSet;
use std::net::SocketAddr;

pub struct Shuffle {
    origin: SocketAddr,
    nodes: HashSet<SocketAddr>,
    ttl: u32
}

impl Shuffle {
  pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
      state.on_shuffle(self).await?;

      Ok(())
  }
}
