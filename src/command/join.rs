use crate::state::PeerState;

#[derive(Debug)]
pub struct Join {}

impl Join {
    pub(crate) async fn apply(&self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_join(self).await?;

        Ok(())
    }
}
