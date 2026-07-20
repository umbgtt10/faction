// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::thread::sleep;
use std::time::Duration;

use faction::peer_state::PeerState;

use crate::node::Node;
use crate::spawn::Spawn;
use crate::timer_delay::TimerDelay;

pub struct Cluster {
    nodes: Vec<Node>,
    spawn: Spawn,
    poll_delay: Duration,
}

impl Cluster {
    #[must_use]
    pub fn new(nodes: Vec<Node>, spawn: Spawn, timer_delay: TimerDelay) -> Self {
        Self {
            nodes,
            spawn,
            poll_delay: timer_delay.duration(),
        }
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

    pub fn poll_until_bootstrapped(&mut self) {
        self.start_all();
        if matches!(self.spawn, Spawn::Process) {
            for node in &mut self.nodes {
                node.wait();
            }
        } else {
            while !self.is_bootstrapped() {
                self.step_all();
                sleep(self.poll_delay);
            }
        }
    }
}

impl Drop for Cluster {
    fn drop(&mut self) {
        for node in &mut self.nodes {
            node.shutdown();
        }
    }
}
