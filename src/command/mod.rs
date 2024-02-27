mod join;
pub use join::Join;

mod shuffle;
pub use shuffle::Shuffle;

mod shuffle_reply;
pub use shuffle_reply::ShuffleReply;

mod forward_join;
pub use forward_join::ForwardJoin;

mod neighbor;
pub use neighbor::Neighbor;

mod disconnect;
pub use disconnect::Disconnect;

use crate::state::PeerState;

pub enum Command {
    Join(Join),
    Shuffle(Shuffle),
    ShuffleReply(ShuffleReply),
    ForwardJoin(ForwardJoin),
    Neighbor(Neighbor),
    Disconnect(Disconnect),
}

impl Command {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        use Command::*;

        match self {
            Join(command) => command.apply(state).await,
            Shuffle(command) => command.apply(state).await,
            ShuffleReply(command) => command.apply(state).await,
            ForwardJoin(command) => command.apply(state).await,
            Neighbor(command) => command.apply(state).await,
            Disconnect(command) => command.apply(state).await,
        }
    }
}
