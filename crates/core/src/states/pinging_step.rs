// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::instance::peer_id::PeerId;
use crate::outcome::Outcome;

pub struct PingingStep<P: PeerId = u64> {
    outcomes: Vec<Outcome<P>>,
    confirmed_peers: Vec<P>,
}

impl<P: PeerId> PingingStep<P> {
    #[must_use]
    pub fn new(confirmed_peers: Vec<P>, peer_id: P) -> Self {
        let is_dup = confirmed_peers.contains(&peer_id);
        let mut new_confirmed_peers = confirmed_peers;
        if !is_dup {
            new_confirmed_peers.push(peer_id);
        }

        let outcome = if is_dup {
            Outcome::DuplicateParticipationIgnored { peer_id }
        } else {
            Outcome::ParticipationAccepted { peer_id }
        };

        Self {
            outcomes: vec![outcome],
            confirmed_peers: new_confirmed_peers,
        }
    }

    #[must_use]
    pub fn confirmed_peers(&self) -> &[P] {
        &self.confirmed_peers
    }

    #[must_use]
    pub fn outcomes(&self) -> &[Outcome<P>] {
        &self.outcomes
    }
}
