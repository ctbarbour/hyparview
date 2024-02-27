use crate::state::PeerState;

pub struct Disconnect {
    alive: bool,
    respond: bool,
}

impl Disconnect {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_disconnect(self).await?;
        Ok(())
    }
}
