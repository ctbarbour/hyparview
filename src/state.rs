use crate::codec::HyParViewCodec;
use crate::messages::*;
use futures::SinkExt;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::Framed;

#[derive(Debug)]
pub struct Config {
    local_peer: SocketAddr,
    active_random_walk_length: u32,
    passive_random_walk_length: u32,
    active_view_capacity: usize,
    passive_view_capacity: usize,
    shuffle_ttl: u32,
    shuffle_active_view_count: usize,
    shuffle_passive_view_count: usize,
    shuffle_interval: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            local_peer: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            active_random_walk_length: 6,
            passive_random_walk_length: 6,
            active_view_capacity: 4,
            passive_view_capacity: 24,
            shuffle_ttl: 4,
            shuffle_active_view_count: 4,
            shuffle_passive_view_count: 4,
            shuffle_interval: 5,
        }
    }
}

#[derive(Debug)]
struct ActivePeer {
    connection: Framed<TcpStream, HyParViewCodec>,

    rx: mpsc::UnboundedReceiver<Box<dyn Message + Send>>,
}

#[derive(Debug)]
pub struct PeerState {
    active_view: HashMap<SocketAddr, mpsc::UnboundedSender<Box<dyn Message + Send>>>,
    passive_view: HashSet<SocketAddr>,
    config: Config,
}

impl PeerState {
    pub fn new(config: Config) -> Self {
        PeerState {
            config: config,
            active_view: HashMap::new(),
            passive_view: HashSet::new(),
        }
    }
}

impl PeerState {
    pub async fn on_join(
        &mut self,
        peer_addr: SocketAddr,
        stream: TcpStream,
    ) -> Result<Option<ActivePeer>, std::io::Error> {
        let active_peer = self.add_peer_to_active_view(peer_addr, stream);

        let forward_join_message = Box::new(ForwardJoinMessage {
            peer: peer_addr,
            ttl: self.config.active_random_walk_length,
        });

        for (peer, tx) in self.active_view.iter() {
            if *peer != peer_addr {
                let _ = tx.send(forward_join_message.clone());
            }
        }

        Ok(active_peer)
    }

    pub async fn on_forward_join(
        &mut self,
        peer_addr: SocketAddr,
        stream: TcpStream,
        message: ForwardJoinMessage,
    ) -> Result<Option<ActivePeer>, std::io::Error> {
        if message.ttl == 0 || self.active_view.len() == 0 {
            let active_peer = self.add_peer_to_active_view(peer_addr, stream);
            return Ok(active_peer);
        }

        if message.ttl == self.config.passive_random_walk_length {
            self.add_peer_to_passive_view(peer_addr);
        }

        let mut rng = thread_rng();

        let mut resevoir = None;
        let mut i = 0;
        for (peer, tx) in self.active_view.iter() {
            if &peer_addr == peer {
                continue;
            }

            i += 1;
            let keep_probability = 1.0 / (i as f64);
            if rng.gen_bool(keep_probability) {
                resevoir = Some(tx);
            }
        }

        if let Some(next) = resevoir {
            let _ = next.send(Box::new(ForwardJoinMessage {
                peer: message.peer,
                ttl: message.ttl - 1,
            }));
        }

        Ok(None)
    }

    pub async fn on_shuffle(
        &mut self,
        sender: SocketAddr,
        message: ShuffleMessage,
    ) -> Result<(), std::io::Error> {
        if message.ttl == 0 {
            let node_count = message.nodes.len();

            let mut resevoir: Vec<SocketAddr> = Vec::with_capacity(node_count);
            let mut i = 0;
            let mut rng = thread_rng();
            for item in self.passive_view.iter() {
                i += 1;
                if i <= node_count {
                    resevoir.push(*item);
                } else {
                    let keep_probability = (node_count as f64) / (i as f64);
                    if rng.gen_bool(keep_probability) {
                        let replace_index = rng.gen_range(0..node_count);
                        resevoir[replace_index] = *item;
                    }
                }
            }

            let socket = TcpStream::connect(message.origin).await?;
            let mut transport = Framed::new(socket, HyParViewCodec::new());
            transport
                .send(Box::new(ShuffleReplyMessage {
                    nodes: resevoir.into_iter().collect(),
                }))
                .await?;
        }

        Ok(())
    }

    pub async fn on_shuffle_reply(
        &mut self,
        message: ShuffleReplyMessage,
    ) -> Result<(), std::io::Error> {
        for peer in message.nodes {
            self.add_peer_to_passive_view(peer);
        }

        Ok(())
    }

    pub async fn on_neighbor(
        &mut self,
        sender: SocketAddr,
        stream: TcpStream,
        message: NeighborMessage,
    ) -> Result<Option<ActivePeer>, std::io::Error> {
        if message.high_priority || !self.is_active_view_full() {
            let active_peer = self.add_peer_to_active_view(sender, stream);
            Ok(active_peer)
        } else {
            Ok(None)
        }
    }

    pub async fn on_disconnect(
        &mut self,
        sender: SocketAddr,
        message: DisconnectMessage,
    ) -> Result<(), std::io::Error> {
        if self.remove_peer_from_active_view(sender, message.respond) {
            // if the active view is not full
            if !self.is_active_view_full() {
                // randomly pick a passive peer
                let mut rng = thread_rng();

                let mut resevoir: Option<SocketAddr> = None;
                let mut i = 0;
                for item in self.passive_view.iter() {
                    i += 1;
                    let keep_probability = 1.0 / (i as f64);
                    if rng.gen_bool(keep_probability) {
                        resevoir = Some(*item);
                    }
                }

                if let Some(peer) = resevoir {
                    let high_priority = self.active_view.is_empty();

                    let socket = TcpStream::connect(peer).await?;
                    let mut transport = Framed::new(socket, HyParViewCodec::new());
                    transport
                        .send(Box::new(NeighborMessage {
                            high_priority: high_priority,
                        }))
                        .await?
                }
            }
        }

        if message.alive {
            self.add_peer_to_passive_view(sender);
        }

        Ok(())
    }

    fn remove_peer_from_active_view(&mut self, peer_addr: SocketAddr, respond: bool) -> bool {
        if let Some(tx) = self.active_view.remove(&peer_addr) {
            if respond {
                let _ = tx.send(Box::new(DisconnectMessage {
                    alive: true,
                    respond: false,
                }));
            }

            self.add_peer_to_passive_view(peer_addr);

            drop(tx); // not sure if we need this.

            true
        } else {
            false
        }
    }

    fn add_peer_to_passive_view(&mut self, peer_addr: SocketAddr) {
        if self.passive_view.contains(&peer_addr)
            || self.active_view.contains_key(&peer_addr)
            || peer_addr == self.config.local_peer
        {
            return;
        }

        if self.is_passive_view_full() {
            let mut rng = thread_rng();

            let mut resevoir: Option<SocketAddr> = None;
            let mut i = 0;
            for item in self.passive_view.iter() {
                i += 1;
                let keep_probability = 1.0 / (i as f64);
                if rng.gen_bool(keep_probability) {
                    resevoir = Some(*item);
                }
            }

            if let Some(item) = resevoir {
                self.passive_view.remove(&item);
            }
        }

        self.passive_view.insert(peer_addr);

        assert!(self.passive_view.len() <= self.config.passive_view_capacity);
    }

    pub fn add_peer_to_active_view(
        &mut self,
        peer_addr: SocketAddr,
        stream: TcpStream,
    ) -> Option<ActivePeer> {
        // Check if the peer is already in the active view or is the current peer
        if self.active_view.contains_key(&peer_addr) || peer_addr == self.config.local_peer {
            return None;
        }

        // If the peer is in the passive view, remove it
        if self.passive_view.contains(&peer_addr) {
            self.passive_view.remove(&peer_addr);
        }

        // If the active view is full, randomly drop an element from the active view
        if self.is_active_view_full() {
            let mut rng = thread_rng();

            let mut resevoir = None;
            let mut i = 0;
            for (k, _) in self.active_view.iter() {
                i += 1;
                let keep_probability = 1.0 / (i as f64);
                if rng.gen_bool(keep_probability) {
                    resevoir = Some(*k);
                }
            }

            if let Some(key) = resevoir {
                self.active_view.remove(&key);
                self.passive_view.insert(peer_addr);
            }
        }

        let (tx, rx) = mpsc::unbounded_channel::<Box<dyn Message + Send>>();
        self.active_view.insert(peer_addr, tx);

        Some(ActivePeer {
            connection: Framed::new(stream, HyParViewCodec::new()),
            rx: rx,
        })
    }

    fn is_passive_view_full(&self) -> bool {
        self.passive_view.len() >= self.config.passive_view_capacity
    }

    fn is_active_view_full(&self) -> bool {
        self.active_view.len() >= self.config.active_view_capacity
    }
}
