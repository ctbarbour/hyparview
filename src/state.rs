use crate::message::*;
use crate::Action;
use rand::seq::IteratorRandom;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet, VecDeque};
use std::default::Default;

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

#[derive(Debug)]
struct State {
    active_view: HashSet<SocketAddr>,
    passive_view: HashSet<SocketAddr>,
    config: Config,
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

impl State {
    pub fn on_join(&mut self, message: JoinMessage, actions: &mut VecDeque<Action>) {
        self.add_peer_to_active_view(message.sender);

        let forward_join = ProtocolMessage::forward_join(
            self.config.local_peer,
            message.sender,
            self.config.active_random_walk_length,
        );

        for peer in self.active_view.iter() {
            if *peer != message.sender {
                actions.push_back(Action::send(*peer, forward_join_message.clone()));
            }
        }
    }

    pub fn on_forward_join(&mut self, message: ForwardJoinMessage, actions: &mut VecDeque<Action>) {
        if ttl == 0 || self.active_view.is_empty() {
            self.add_peer_to_active_view(message.sender);
            return;
        }

        if ttl == self.config.passive_random_walk_length {
            self.add_peer_to_passive_view(message.sender);
        }

        let mut rng = thread_rng();

        match self
            .active_view
            .iter()
            .filter(|key| {
                if *key == message.sender {
                    Some(key)
                } else {
                    None
                }
            })
            .choose(&mut rng)
        {
            Some(next) => {
                let forward_join = ProtocolMessage::forward_join(
                    self.config.local_peer,
                    message.sender,
                    message.ttl - 1,
                );
                actions.push_back(Action::Send(next, forward_join));
            }
            None => {
                self.add_peer_to_active_view(message.sender);
            }
        }
    }

    pub fn on_shuffle(&mut self, message: ShuffleMessage, actions: &mut VecDeque<Action>) {
        if message.ttl == 0 {
            let node_count = nodes.len();
            let mut rng = thread_rng();

            let shuffled_nodes = self
                .passive_view
                .iter()
                .choose_multiple(&mut rng, node_count);

            let shuffle_reply = ProtocolMessage::shuffle_reply(
                self.config.local_peer,
                shuffled_nodes.into_iter().collect(),
            );
            actions.push_back(Actions::send(message.origin, shuffle_reply));
        }
    }

    pub fn on_shuffle_reply(
        &mut self,
        message: ShuffleReplyMessage,
        actions: &mut VecDeque<Action>,
    ) {
        for peer in message.nodes {
            self.add_peer_to_passive_view(peer);
        }
    }

    pub fn on_neighbor(
        &mut self,
        message: NeighborMessage,
        actions: &mut VecDeque<Action>,
    ) -> Result<(), std::io::Error> {
        if message.high_priority || !self.is_active_view_full() {
            self.add_peer_to_active_view(message.sender);
        }
    }

    pub fn on_disconnect(&mut self, message: DisconnectMessage, actions: &mut VecDeque<Action>) {
        if self.active_view.remove(message.sender) {
            if message.respond {
                let disconnect_message =
                    ProtocolMessage::disconnect(self.config.local_peer, true, false);
                actions.push_back(Action::send(message.sender, disconnect_message));
            }

            // if the active view is not full
            if !self.is_active_view_full() {
                // randomly pick a passive peer
                let mut rng = thread_rng();

                if let Some(peer) = self.passive_view.iter().choose(&mut rng) {
                    let high_priority = self.active_view.is_empty();

                    let neighbor_message =
                        ProtocolMessage::neighbor(self.config.local_peer, high_priority);

                    actions.push_back(Action::send(peer, neighbor_message));
                }
            }
        }

        if alive {
            self.add_peer_to_passive_view(message.sender);
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
            if let Some(peer) = self.passive_view.iter().choose(&mut rng) {
                self.passive_view.remove(&peer);
            }
        }

        self.passive_view.insert(peer_addr);
    }

    fn add_peer_to_active_view(&mut self, peer_addr: SocketAddr) {
        // Check if the peer is already in the active view or is the current peer
        if self.active_view.contains_key(&peer_addr) || peer_addr == self.config.local_peer {
            return;
        }

        // If the peer is in the passive view, remove it
        if self.passive_view.contains(&peer_addr) {
            self.passive_view.remove(&peer_addr);
        }

        // If the active view is full, randomly drop an element from the active view
        if self.is_active_view_full() {
            let mut rng = thread_rng();

            if let Some(peer) = self.active_view.iter().choose(&mut rng) {
                self.active_view.remove(&peer);
                self.passive_view.insert(peer_addr);
            }
        }

        self.active_view.insert(peer_addr);
    }

    fn is_passive_view_full(&self) -> bool {
        self.passive_view.len() >= self.config.passive_view_capacity
    }

    fn is_active_view_full(&self) -> bool {
        self.active_view.len() >= self.config.active_view_capacity
    }
}
