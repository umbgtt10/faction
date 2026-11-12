// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::conclusion::Conclusion;
use crate::outcome::Outcome;
use crate::quorum_policy::QuorumPolicy;
use crate::types::PeerId;

pub struct LocalCompletionStep {
    outcomes: Vec<Outcome>,
    confirmed_peers: Vec<PeerId>,
    is_quorum: bool,
}

impl LocalCompletionStep {
    #[must_use]
    pub fn new(confirmed_peers: Vec<PeerId>, peer_id: PeerId, quorum_policy: QuorumPolicy) -> Self {
        let was_present = confirmed_peers.contains(&peer_id);
        let mut new_confirmed_peers = confirmed_peers;
        if !was_present {
            new_confirmed_peers.push(peer_id);
        }

        let is_quorum = quorum_policy.is_satisfied(new_confirmed_peers.len());
        let mut outcomes = vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ];

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
    pub fn confirmed_peers(&self) -> &[PeerId] {
        &self.confirmed_peers
    }

    #[must_use]
    pub fn is_quorum(&self) -> bool {
        self.is_quorum
    }

    #[must_use]
    pub fn outcomes(&self) -> &[Outcome] {
        &self.outcomes
    }
}
