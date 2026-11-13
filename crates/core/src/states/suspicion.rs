// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;
use core::iter::once;

use crate::members::Members;
use crate::miss_count::MissCount;
use crate::outcome::Outcome;
use crate::quorum_policy::QuorumPolicy;
use crate::types::PeerId;

// Whom a concluded node suspects, in the order suspected, and how many periods
// running each member not suspected has missed, in the members' order. A
// member heard from is alive again, and has missed nothing.
#[derive(Debug, Clone, Default)]
pub struct Suspicion {
    suspected: Vec<PeerId>,
    misses: Vec<MissCount>,
}

impl Suspicion {
    #[must_use]
    pub fn heard(&self, peer_id: PeerId) -> (Outcome, Self) {
        let outcome = if self.is_suspected(peer_id) {
            Outcome::PeerRevived { peer_id }
        } else {
            Outcome::AliveNoted { peer_id }
        };
        (outcome, self.forgotten(peer_id))
    }

    #[must_use]
    pub fn holds_quorum(&self, members: &Members, quorum_policy: QuorumPolicy) -> bool {
        let alive = members
            .as_slice()
            .iter()
            .filter(|member| !self.is_suspected(**member))
            .count();
        quorum_policy.is_satisfied(alive)
    }

    #[must_use]
    pub fn is_suspected(&self, peer_id: PeerId) -> bool {
        self.suspected.contains(&peer_id)
    }

    // One more period missed: counted below the threshold, suspected at it.
    #[must_use]
    pub fn missed_period(
        &self,
        peer_id: PeerId,
        members: &Members,
        threshold: usize,
    ) -> (Outcome, Self) {
        let misses = self
            .misses
            .iter()
            .find(|miss| miss.peer_id == peer_id)
            .map_or(0, |miss| miss.misses)
            + 1;
        if misses < threshold {
            (
                Outcome::MissCounted { peer_id, misses },
                Self {
                    misses: self.counted(peer_id, misses, members),
                    ..self.clone()
                },
            )
        } else {
            let forgotten = self.forgotten(peer_id);
            (
                Outcome::PeerSuspected { peer_id },
                Self {
                    suspected: forgotten
                        .suspected
                        .iter()
                        .copied()
                        .chain(once(peer_id))
                        .collect(),
                    ..forgotten
                },
            )
        }
    }

    #[must_use]
    pub fn misses(&self) -> &[MissCount] {
        &self.misses
    }

    #[must_use]
    pub fn suspected(&self) -> &[PeerId] {
        &self.suspected
    }

    // The missed periods with `peer_id`'s now `misses`, in the members' order.
    fn counted(&self, peer_id: PeerId, misses: usize, members: &Members) -> Vec<MissCount> {
        members
            .as_slice()
            .iter()
            .filter_map(|member| {
                if *member == peer_id {
                    Some(MissCount { peer_id, misses })
                } else {
                    self.misses
                        .iter()
                        .copied()
                        .find(|miss| miss.peer_id == *member)
                }
            })
            .collect()
    }

    fn forgotten(&self, peer_id: PeerId) -> Self {
        Self {
            suspected: self
                .suspected
                .iter()
                .copied()
                .filter(|suspect| *suspect != peer_id)
                .collect(),
            misses: self
                .misses
                .iter()
                .copied()
                .filter(|miss| miss.peer_id != peer_id)
                .collect(),
        }
    }
}
