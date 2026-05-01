// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::thread::sleep;
use std::time::Duration;

use faction::peer_state::PeerState;

use crate::node::Node;

pub struct Cluster {
    nodes: Vec<Node>,
}

impl Cluster {
    #[must_use]
    pub fn new(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }

    pub fn start_all(&mut self) {
        for node in &self.nodes {
            node.start();
        }
    }

    pub fn step_all(&mut self) {
        for node in &self.nodes {
            node.step();
        }
    }

    #[must_use]
    pub fn is_bootstrapped(&self) -> bool {
        self.nodes
            .iter()
            .all(|node| node.peer_state() == PeerState::Bootstrapped)
    }

    pub fn poll_until_bootstrapped(&mut self, delay_ms: u64) {
        self.start_all();
        while !self.is_bootstrapped() {
            self.step_all();
            sleep(Duration::from_millis(delay_ms));
        }
    }
}
