// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::instance::peer_id::PeerId;
use crate::outcome::Outcome;

// A member's readiness before the node's own completion: accepted once, the
// peer the last collecting peer, a repeat ignored. Nothing concludes before
// the node completes.
pub struct ReadyBeforeCompletionStep<P: PeerId = u64> {
    outcomes: Vec<Outcome<P>>,
    collecting_peers: Vec<P>,
}

impl<P: PeerId> ReadyBeforeCompletionStep<P> {
    #[must_use]
    pub fn new(collecting_peers: Vec<P>, peer_id: P) -> Self {
        if collecting_peers.contains(&peer_id) {
            Self {
                outcomes: vec![Outcome::DuplicateReadyIgnored { peer_id }],
                collecting_peers,
            }
        } else {
            Self {
                outcomes: vec![Outcome::ReadyAccepted { peer_id }],
                collecting_peers: collecting_peers.into_iter().chain([peer_id]).collect(),
            }
        }
    }

    #[must_use]
    pub fn collecting_peers(&self) -> &[P] {
        &self.collecting_peers
    }

    #[must_use]
    pub fn outcomes(&self) -> &[Outcome<P>] {
        &self.outcomes
    }
}
