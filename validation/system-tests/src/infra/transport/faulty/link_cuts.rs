// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::Mutex;

use faction::types::PeerId;

// The peers a node's links are cut from, while a test keeps them cut: nothing
// is sent to them, and nothing they send is received. A clone shares the cuts,
// so the node that is told to cut a link and the transport that drops its
// messages hold the same ones.
#[derive(Debug, Clone, Default)]
pub struct LinkCuts {
    cut: Arc<Mutex<BTreeSet<PeerId>>>,
}

impl LinkCuts {
    pub fn cut(&self, peer_id: PeerId) {
        self.cut.lock().unwrap().insert(peer_id);
    }

    #[must_use]
    pub fn is_cut(&self, peer_id: PeerId) -> bool {
        self.cut.lock().unwrap().contains(&peer_id)
    }

    pub fn mend(&self, peer_id: PeerId) {
        self.cut.lock().unwrap().remove(&peer_id);
    }
}
