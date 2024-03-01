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

use crate::PeerState;
use serde::{Deserialize, Serialize};

use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolMessage {
    Join(Join),
    Shuffle(Shuffle),
    ShuffleReply(ShuffleReply),
    ForwardJoin(ForwardJoin),
    Neighbor(Neighbor),
    Disconnect(Disconnect),
}

impl ProtocolMessage {
    pub(crate) async fn apply(self, state: &PeerState) -> Result<(), std::io::Error> {
        use ProtocolMessage::*;

        match self {
            Join(message) => message.apply(state).await,
            Shuffle(message) => message.apply(state).await,
            ShuffleReply(message) => message.apply(state).await,
            ForwardJoin(message) => message.apply(state).await,
            Neighbor(message) => message.apply(state).await,
            Disconnect(message) => message.apply(state).await,
        }
    }

    pub(crate) fn sender(&self) -> SocketAddr {
        use ProtocolMessage::*;

        match self {
            Join(message) => message.sender,
            Shuffle(message) => message.sender,
            ShuffleReply(message) => message.sender,
            ForwardJoin(message) => message.sender,
            Neighbor(message) => message.sender,
            Disconnect(message) => message.sender,
        }
    }

    pub(crate) fn join(sender: SocketAddr) -> Self {
        ProtocolMessage::Join(Join { sender: sender })
    }

    pub(crate) fn forward_join(sender: SocketAddr, peer: SocketAddr, ttl: u32) -> Self {
        ProtocolMessage::ForwardJoin(ForwardJoin {
            sender: sender,
            peer: peer,
            ttl: ttl,
        })
    }

    pub(crate) fn neighbor(sender: SocketAddr, high_priority: bool) -> Self {
        ProtocolMessage::Neighbor(Neighbor {
            sender: sender,
            high_priority,
        })
    }

    pub(crate) fn disconnect(sender: SocketAddr, alive: bool, respond: bool) -> Self {
        ProtocolMessage::Disconnect(Disconnect {
            sender,
            alive,
            respond,
        })
    }

    pub(crate) fn shuffle_reply(sender: SocketAddr, nodes: Vec<SocketAddr>) -> Self {
        ProtocolMessage::ShuffleReply(ShuffleReply { sender, nodes })
    }
}
