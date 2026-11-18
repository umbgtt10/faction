// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::conclusion::Conclusion;
use crate::instance::peer_id::PeerId;
use crate::outcome::Outcome;
use crate::quorum_policy::QuorumPolicy;

pub struct ReadyStep<P: PeerId = u64> {
    outcomes: Vec<Outcome<P>>,
    confirmed_peers: Vec<P>,
    is_quorum: bool,
}

impl<P: PeerId> ReadyStep<P> {
    #[must_use]
    pub fn new(confirmed_peers: Vec<P>, peer_id: P, quorum_policy: QuorumPolicy) -> Self {
        let is_dup = confirmed_peers.contains(&peer_id);
        let confirmed_new = !is_dup;

        let mut new_confirmed_peers = confirmed_peers;
        if confirmed_new {
            new_confirmed_peers.push(peer_id);
        }

        let is_quorum = confirmed_new && quorum_policy.is_satisfied(new_confirmed_peers.len());

        let outcome = if is_dup {
            Outcome::DuplicateReadyIgnored { peer_id }
        } else {
            Outcome::ReadyAccepted { peer_id }
        };

        let mut outcomes = vec![outcome];
        if is_quorum {
            outcomes.push(Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            });
        }

        Self {
            outcomes,
            confirmed_peers: new_confirmed_peers,
            is_quorum,
        }
    }

    #[must_use]
    pub fn confirmed_peers(&self) -> &[P] {
        &self.confirmed_peers
    }

    #[must_use]
    pub fn is_quorum(&self) -> bool {
        self.is_quorum
    }

    #[must_use]
    pub fn outcomes(&self) -> &[Outcome<P>] {
        &self.outcomes
    }
}
