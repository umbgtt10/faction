// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

use faction::types::PeerId;

use crate::infra::node_faults::NodeFaults;

// Every node's faults, by peer: a recipe puts each node it builds behind its
// own, and whoever runs the cluster switches them while it runs. A clone
// holds the same board, and a node never set is faithful.
#[derive(Debug, Clone, Default)]
pub struct FaultBoard {
    nodes: Arc<Mutex<BTreeMap<PeerId, NodeFaults>>>,
}

impl FaultBoard {
    #[must_use]
    pub fn node(&self, peer_id: PeerId) -> NodeFaults {
        self.nodes
            .lock()
            .unwrap()
            .entry(peer_id)
            .or_default()
            .clone()
    }
}
