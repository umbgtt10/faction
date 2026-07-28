// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Members {
    peers: Vec<PeerId>,
}

impl Members {
    #[must_use]
    pub fn new(peers: Vec<PeerId>) -> Self {
        Self { peers }
    }

    #[must_use]
    pub fn is_member(&self, peer_id: PeerId) -> bool {
        self.peers.contains(&peer_id)
    }

    #[must_use]
    pub fn with_admitted(&self, peer_id: PeerId) -> Self {
        let mut peers = self.peers.clone();
        if !peers.contains(&peer_id) {
            peers.push(peer_id);
        }
        Self { peers }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    #[must_use]
    pub fn as_slice(&self) -> &[PeerId] {
        &self.peers
    }
}
