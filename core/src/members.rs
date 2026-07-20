// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

use crate::types::PeerId;

#[derive(Clone)]
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
}
