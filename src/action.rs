use crate::message::ProtocolMessage;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum Action {
    Send {
        destination: SocketAddr,
        message: ProtocolMessage,
    },

    Disconnect {
        peer: SocketAddr,
    },
}

impl Action {
    pub(crate) fn send(destination: SocketAddr, message: ProtocolMessage) -> Self {
        Action::Send {
            destination,
            message,
        }
    }

    pub(crate) fn disconnect(peer: SocketAddr) -> Self {
        Action::Disconnect { peer }
    }
}
