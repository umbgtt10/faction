// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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

    #[must_use]
    pub fn is_bootstrapped(&self) -> bool {
        self.nodes
            .iter()
            .all(|node| node.peer_state() == PeerState::Bootstrapped)
    }
}
