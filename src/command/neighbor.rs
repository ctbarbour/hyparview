use crate::state::PeerState;

pub struct Neighbor {
    high_priority: bool
}

impl Neighbor {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_neighbor(self).await?;

        Ok(())
    }
}
