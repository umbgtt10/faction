// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::command::Command;
use crate::instance::peer_id::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Members<P: PeerId = u64> {
    peers: Vec<P>,
}

impl<P: PeerId> Members<P> {
    #[must_use]
    pub fn new(peers: Vec<P>) -> Self {
        Self { peers }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[P] {
        &self.peers
    }

    #[must_use]
    pub fn is_member(&self, peer_id: P) -> bool {
        self.peers.contains(&peer_id)
    }

    #[must_use]
    pub fn non_member_observed(&self, command: &Command<P>) -> Option<P> {
        match command {
            Command::ParticipationObserved { peer_id, .. }
            | Command::ReadyObserved { peer_id, .. }
                if !self.is_member(*peer_id) =>
            {
                Some(*peer_id)
            }
            _ => None,
        }
    }

    // The peers given that are members, in the order given.
    #[must_use]
    pub fn retained(&self, peers: &[P]) -> Vec<P> {
        peers
            .iter()
            .copied()
            .filter(|peer| self.is_member(*peer))
            .collect()
    }

    #[must_use]
    pub fn with_admitted(&self, peer_id: P) -> Option<Self> {
        if self.is_member(peer_id) {
            None
        } else {
            let mut peers = self.peers.clone();
            peers.push(peer_id);
            Some(Self { peers })
        }
    }

    // The members but `peer_id`; none if it is not one.
    #[must_use]
    pub fn without(&self, peer_id: P) -> Option<Self> {
        self.is_member(peer_id).then(|| Self {
            peers: self
                .peers
                .iter()
                .copied()
                .filter(|member| *member != peer_id)
                .collect(),
        })
    }
}
