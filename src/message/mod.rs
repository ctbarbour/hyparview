mod join;
pub use join::JoinMessage;

mod shuffle;
pub use shuffle::ShuffleMessage;

mod shuffle_reply;
pub use shuffle_reply::ShuffleReplyMessage;

mod forward_join;
pub use forward_join::ForwardJoinMessage;

mod neighbor;
pub use neighbor::NeighborMessage;

mod disconnect;
pub use disconnect::DisconnectMessage;

use crate::PeerState;

pub enum ProtocolMessage {
    Join(JoinMessage),
    Shuffle(ShuffleMessage),
    ShuffleReply(ShuffleReplyMessage),
    ForwardJoin(ForwardJoinMessage),
    Neighbor(NeighborMessage),
    Disconnect(DisconnectMessage),
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

    pub(crate) fn sender() -> SocketAddr {
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
        ProtocolMessage::JoinMessage { sender: sender }
    }

    pub(crate) fn forward_join(sender: SocketAddr, peer: SocketAddr, ttl: u32) -> Self {
        ProtocolMessage::ForwardJoinMessage {
            sender: sender,
            peer: peer,
            ttl: ttl,
        }
    }

    pub(crate) fn neighbor(
        sender: SocketAddr,
        origin: SocketAddr,
        nodes: HashSet<SocketAddr>,
        ttl: u32,
    ) -> Self {
        ProtocolMessage::NeighborMessage {
            sender: sender,
            origin: origin,
            nodes: nodes,
            ttl: ttl,
        }
    }
}
