use crate::PeerState;

#[derive(Debug)]
pub struct JoinMessage {
    pub sender: SocketAddr,
}

impl JoinMessage {
    pub(crate) async fn apply(&self, state: &PeerState) -> Result<(), std::io::Error> {
        state.on_join(self).await;
        Ok(())
    }
}
