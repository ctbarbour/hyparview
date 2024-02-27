use bytes::{Buf, BufMut, BytesMut};
use std::collections::HashSet;
use std::default::Default;
use std::io::Error;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

// Debug is a super trait to get Box<dyn Message> to debug fmt properly
pub trait Message: std::fmt::Debug {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), std::io::Error>;
    fn decode(&mut self, buffer: &mut BytesMut) -> Result<(), std::io::Error>;
    async fn accept(&self, handler: &mut dyn Handler) -> Result<(), std::io::Error>;
}

pub trait Handler {
    async fn on_join(&mut self, message: &JoinMessage) -> Result<(), std::io::Error>;
    async fn on_shuffle(&mut self, message: &ShuffleMessage) -> Result<(), std::io::Error>;
    async fn on_forward_join(&mut self, message: &ForwardJoinMessage) -> Result<(), std::io::Error>;
    async fn on_shuffle_reply(&mut self, message: &ShuffleReplyMessage) -> Result<(), std::io::Error>;
    async fn on_neighbor(&mut self, message: &NeighborMessage) -> Result<(), std::io::Error>;
    async fn on_disconnect(&mut self, message: &DisconnectMessage) -> Result<(), std::io::Error>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct JoinMessage {}

#[derive(Debug, PartialEq, Clone)]
pub struct ShuffleMessage {
    pub origin: SocketAddr,
    pub nodes: HashSet<SocketAddr>,
    pub ttl: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ForwardJoinMessage {
    pub peer: SocketAddr,
    pub ttl: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ShuffleReplyMessage {
    pub nodes: HashSet<SocketAddr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NeighborMessage {
    pub high_priority: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DisconnectMessage {
    pub alive: bool,
    pub respond: bool,
}

impl Message for DisconnectMessage {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), std::io::Error> {
        buffer.put_u8(5);
        buffer.put_u8(self.alive.into());
        buffer.put_u8(self.respond.into());

        Ok(())
    }

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<(), std::io::Error> {
        self.alive = match buffer.get_u8() {
            0 => false,
            _ => true,
        };

        self.respond = match buffer.get_u8() {
            0 => false,
            _ => true,
        };

        Ok(())
    }

    async fn accept(&self, handler: &mut dyn Handler) -> Result<(), std::io::Error> {
        handler.on_disconnect(self)
    }
}

impl Message for NeighborMessage {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), std::io::Error> {
        buffer.put_u8(6);
        buffer.put_u8(self.high_priority.into());

        Ok(())
    }

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<(), std::io::Error> {
        self.high_priority = match buffer.get_u8() {
            0 => false,
            _ => true,
        };

        Ok(())
    }

    async fn accept(&self, handler: &mut dyn Handler) -> Result<(), std::io::Error> {
        handler.on_neighbor(self)
    }
}

impl Default for ShuffleMessage {
    fn default() -> Self {
        ShuffleMessage {
            origin: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            nodes: Default::default(),
            ttl: 0,
        }
    }
}

impl Message for JoinMessage {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), std::io::Error> {
        buffer.put_u8(1);
        Ok(())
    }

    fn decode(&mut self, _buffer: &mut BytesMut) -> Result<(), std::io::Error> {
        Ok(())
    }

    async fn accept(&self, handler: &mut dyn Handler) -> Result<(), std::io::Error> {
        handler.on_join(self)
    }
}

impl Message for ShuffleMessage {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), std::io::Error> {
        buffer.put_u8(2);
        buffer.extend_from_slice(&self.origin.to_string().as_bytes());
        buffer.put_u8(0);
        buffer.extend_from_slice(&(self.nodes.len() as u32).to_be_bytes());
        for node in &self.nodes {
            buffer.extend_from_slice(&node.to_string().as_bytes());
            buffer.put_u8(0);
        }
        buffer.extend_from_slice(&self.ttl.to_be_bytes());
        Ok(())
    }

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<(), Error> {
        if buffer.len() < 8 {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Insufficient data for ShuffleMessage",
            ));
        }

        let origin_str =
            String::from_utf8_lossy(&buffer[..buffer.iter().position(|b| *b == 0).unwrap()]);
        self.origin = match origin_str.parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(_) => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid origin address",
                ))
            }
        };
        buffer.advance(buffer.iter().position(|b| *b == 0).unwrap() + 1);
        self.nodes.clear();
        let node_count = buffer.get_u32();
        if buffer.len()
            < (node_count as usize * (mem::size_of::<SocketAddr>() + 1))
                .try_into()
                .unwrap()
        {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Insufficient data for nodes",
            ));
        }
        for _ in 0..node_count {
            let node_str =
                String::from_utf8_lossy(&buffer[..buffer.iter().position(|b| *b == 0).unwrap()]);
            self.nodes.insert(match node_str.parse::<SocketAddr>() {
                Ok(addr) => addr,
                Err(_) => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid node address",
                    ))
                }
            });
            buffer.advance(buffer.iter().position(|b| *b == 0).unwrap() + 1);
        }
        self.ttl = buffer.get_u32();
        Ok(())
    }

    async fn accept(&self, handler: &mut dyn Handler) -> Result<(), std::io::Error> {
        handler.on_shuffle(self).await
    }
}

impl Message for ForwardJoinMessage {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), std::io::Error> {
        buffer.put_u8(3);
        buffer.extend_from_slice(&self.peer.to_string().as_bytes());
        buffer.put_u8(0);
        buffer.put_u32(self.ttl);
        Ok(())
    }

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<(), std::io::Error> {
        let peer_str =
            String::from_utf8_lossy(&buffer[..buffer.iter().position(|b| *b == 0).unwrap()]);
        self.peer = match peer_str.parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(_) => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid origin address",
                ))
            }
        };

        self.ttl = buffer.get_u32();
        Ok(())
    }

    async fn accept(&self, handler: &mut dyn Handler) -> Result<(), std::io::Error> {
        handler.on_forward_join(self)
    }
}

impl Message for ShuffleReplyMessage {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), std::io::Error> {
        buffer.put_u8(4);
        buffer.extend_from_slice(&(self.nodes.len() as u32).to_be_bytes());
        for node in &self.nodes {
            buffer.extend_from_slice(&node.to_string().as_bytes());
            buffer.put_u8(0);
        }

        Ok(())
    }

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<(), std::io::Error> {
        self.nodes.clear();
        let node_count = buffer.get_u32();
        if buffer.len()
            < (node_count as usize * (mem::size_of::<SocketAddr>() + 1))
                .try_into()
                .unwrap()
        {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Insufficient data for nodes",
            ));
        }
        for _ in 0..node_count {
            let node_str =
                String::from_utf8_lossy(&buffer[..buffer.iter().position(|b| *b == 0).unwrap()]);
            self.nodes.insert(match node_str.parse::<SocketAddr>() {
                Ok(addr) => addr,
                Err(_) => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid node address",
                    ))
                }
            });
            buffer.advance(buffer.iter().position(|b| *b == 0).unwrap() + 1);
        }

        Ok(())
    }

    async fn accept(&self, handler: &mut dyn Handler) -> Result<(), std::io::Error> {
        handler.on_shuffle_reply(self)
    }
}
