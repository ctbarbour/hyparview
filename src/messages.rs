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
}

#[derive(Debug, PartialEq)]
pub struct JoinMessage {}

#[derive(Debug, PartialEq)]
pub struct ShuffleMessage {
    pub origin: SocketAddr,
    pub nodes: HashSet<SocketAddr>,
    pub ttl: u32,
}

#[derive(Debug, PartialEq)]
pub struct ForwardJoinMessage {
    peer: SocketAddr,
    ttl: u32,
}

#[derive(Debug, PartialEq)]
pub struct ShuffleReplyMessage {
    nodes: HashSet<SocketAddr>,
}

#[derive(Debug, PartialEq)]
pub struct NeighborMessage {
    high_priority: bool,
}

#[derive(Debug, PartialEq)]
pub struct DisconnectMessage {
    alive: bool,
    respond: bool,
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
}
