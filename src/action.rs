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
    use crate::Action::*;

    pub(crate) fn send(destination: SocketAddr, message: ProtocolMessage) -> Self {
        Send { destination, message, }
    }

    pub(crate) fn disconnect(peer: SocketAddr) -> Self {
        Disconnect { peer }
    }
}
