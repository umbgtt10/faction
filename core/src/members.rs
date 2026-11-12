// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::command::Command;
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
    pub fn as_slice(&self) -> &[PeerId] {
        &self.peers
    }

    #[must_use]
    pub fn is_member(&self, peer_id: PeerId) -> bool {
        self.peers.contains(&peer_id)
    }

    #[must_use]
    pub fn non_member_observed(&self, command: &Command) -> Option<PeerId> {
        match command {
            Command::ParticipationObserved { peer_id } | Command::ReadyObserved { peer_id }
                if !self.is_member(*peer_id) =>
            {
                Some(*peer_id)
            }
            _ => None,
        }
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
