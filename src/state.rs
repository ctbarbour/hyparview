use crate::message::*;
use crate::Action;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::{HashSet, VecDeque};
use std::default::Default;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::Mutex;

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

#[derive(Debug, Clone)]
pub(crate) struct PeerState {
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
}

#[derive(Debug)]
pub struct State {
    active_view: HashSet<SocketAddr>,
    passive_view: HashSet<SocketAddr>,
    config: Config,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            local_peer: SocketAddr::new("127.0.0.1".parse().unwrap(), 8080),
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

impl PeerState {
    pub(crate) fn new(config: Config) -> PeerState {
        PeerState {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    active_view: HashSet::new(),
                    passive_view: HashSet::new(),
                    config: config,
                }),
            }),
        }
    }

    pub(crate) async fn on_join(&self, message: Join) -> Result<(), std::io::Error> {
        let mut state = self.shared.state.lock().await;
        let mut actions = VecDeque::new();
        state.on_join(message, &mut actions);
        Ok(())
    }

    pub(crate) async fn on_shuffle(&self, message: Shuffle) -> Result<(), std::io::Error> {
        let mut state = self.shared.state.lock().await;
        let mut actions = VecDeque::new();
        state.on_shuffle(message, &mut actions);
        Ok(())
    }

    pub(crate) async fn on_forward_join(&self, message: ForwardJoin) -> Result<(), std::io::Error> {
        let mut state = self.shared.state.lock().await;
        let mut actions = VecDeque::new();
        state.on_forward_join(message, &mut actions);
        Ok(())
    }

    pub(crate) async fn on_shuffle_reply(
        &self,
        message: ShuffleReply,
    ) -> Result<(), std::io::Error> {
        let mut state = self.shared.state.lock().await;
        let mut actions = VecDeque::new();
        state.on_shuffle_reply(message, &mut actions);
        Ok(())
    }

    pub(crate) async fn on_neighbor(&self, message: Neighbor) -> Result<(), std::io::Error> {
        let mut state = self.shared.state.lock().await;
        let mut actions = VecDeque::new();
        state.on_neighbor(message, &mut actions);
        Ok(())
    }

    pub(crate) async fn on_disconnect(&self, message: Disconnect) -> Result<(), std::io::Error> {
        let mut state = self.shared.state.lock().await;
        let mut actions = VecDeque::new();
        state.on_disconnect(message, &mut actions);
        Ok(())
    }
}

impl State {
    pub fn on_join(&mut self, message: Join, actions: &mut VecDeque<Action>) {
        self.add_peer_to_active_view(message.sender);

        let forward_join = ProtocolMessage::forward_join(
            self.config.local_peer,
            message.sender,
            self.config.active_random_walk_length,
        );

        for peer in self.active_view.iter() {
            if *peer != message.sender {
                actions.push_back(Action::send(*peer, forward_join.clone()));
            }
        }
    }

    pub fn on_forward_join(&mut self, message: ForwardJoin, actions: &mut VecDeque<Action>) {
        if message.ttl == 0 || self.active_view.is_empty() {
            self.add_peer_to_active_view(message.sender);
            return;
        }

        if message.ttl == self.config.passive_random_walk_length {
            self.add_peer_to_passive_view(message.sender);
        }

        if !self.active_view.is_empty() {
            if let Some(peer) = Self::select_random_peer(&self.active_view) {
                let forward_join = ProtocolMessage::forward_join(
                    self.config.local_peer,
                    message.sender,
                    message.ttl - 1,
                );
                actions.push_back(Action::send(peer, forward_join));
            }
        } else {
            self.add_peer_to_active_view(message.sender);
        }
    }

    pub fn on_shuffle(&mut self, message: Shuffle, actions: &mut VecDeque<Action>) {
        if message.ttl == 0 {
            let node_count = message.nodes.len();
            let shuffled_peers = Self::select_random_n_peers(&self.passive_view, node_count);
            let shuffle_reply =
                ProtocolMessage::shuffle_reply(self.config.local_peer, shuffled_peers);
            actions.push_back(Action::send(message.origin, shuffle_reply));
        }
    }

    pub fn on_shuffle_reply(&mut self, message: ShuffleReply, actions: &mut VecDeque<Action>) {
        for peer in message.nodes {
            self.add_peer_to_passive_view(peer);
        }
    }

    pub fn on_neighbor(
        &mut self,
        message: Neighbor,
        actions: &mut VecDeque<Action>,
    ) -> Result<(), std::io::Error> {
        if message.high_priority || !self.is_active_view_full() {
            self.add_peer_to_active_view(message.sender);
        }

        Ok(())
    }

    pub fn on_disconnect(&mut self, message: Disconnect, actions: &mut VecDeque<Action>) {
        if self.active_view.remove(&message.sender) {
            if message.respond {
                let disconnect_message =
                    ProtocolMessage::disconnect(self.config.local_peer, true, false);
                actions.push_back(Action::send(message.sender, disconnect_message));
            }

            // if the active view is not full
            if !self.is_active_view_full() {
                // randomly pick a passive peer
                if let Some(peer) = Self::select_random_peer(&self.passive_view) {
                    let high_priority = self.active_view.is_empty();
                    let neighbor_message =
                        ProtocolMessage::neighbor(self.config.local_peer, high_priority);
                    actions.push_back(Action::send(peer, neighbor_message));
                }
            }
        }

        if message.alive {
            self.add_peer_to_passive_view(message.sender);
        }
    }

    fn add_peer_to_passive_view(&mut self, peer_addr: SocketAddr) {
        if self.passive_view.contains(&peer_addr)
            || self.active_view.contains(&peer_addr)
            || peer_addr == self.config.local_peer
        {
            return;
        }

        if self.is_passive_view_full() {
            if let Some(peer) = Self::select_random_peer(&self.passive_view) {
                self.passive_view.remove(&peer);
            }
        }

        self.passive_view.insert(peer_addr);
    }

    fn add_peer_to_active_view(&mut self, peer_addr: SocketAddr) {
        // Check if the peer is already in the active view or is the current peer
        if self.active_view.contains(&peer_addr) || peer_addr == self.config.local_peer {
            return;
        }

        // If the peer is in the passive view, remove it
        if self.passive_view.contains(&peer_addr) {
            self.passive_view.remove(&peer_addr);
        }

        // If the active view is full, randomly drop an element from the active view
        if self.is_active_view_full() {
            if let Some(peer) = Self::select_random_peer(&self.active_view) {
                self.active_view.remove(&peer);
                self.passive_view.insert(peer_addr);
            }
        }

        self.active_view.insert(peer_addr);
    }

    fn select_random_peer(set: &HashSet<SocketAddr>) -> Option<SocketAddr> {
        let mut rng = thread_rng();
        let vec: Vec<_> = set.iter().copied().collect();
        vec.choose(&mut rng).copied()
    }

    fn select_random_n_peers(set: &HashSet<SocketAddr>, n: usize) -> Vec<SocketAddr> {
        let mut rng = thread_rng();
        let vec: Vec<_> = set.iter().copied().collect();
        vec.choose_multiple(&mut rng, n).copied().collect()
    }

    fn is_passive_view_full(&self) -> bool {
        self.passive_view.len() >= self.config.passive_view_capacity
    }

    fn is_active_view_full(&self) -> bool {
        self.active_view.len() >= self.config.active_view_capacity
    }
}
