// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;
use core::iter::once;

use crate::instance::peer_id::PeerId;
use crate::members::Members;
use crate::miss_count::MissCount;
use crate::outcome::Outcome;
use crate::quorum_policy::QuorumPolicy;

// Whom a concluded node suspects, in the order suspected, and how many periods
// running each member not suspected has missed, in the members' order. A
// member heard from is alive again, and has missed nothing.
#[derive(Debug, Clone)]
pub struct Suspicion<P: PeerId = u64> {
    suspected: Vec<P>,
    misses: Vec<MissCount<P>>,
}

impl<P: PeerId> Suspicion<P> {
    #[must_use]
    pub fn heard(&self, peer_id: P) -> (Outcome<P>, Self) {
        let outcome = if self.is_suspected(peer_id) {
            Outcome::PeerRevived { peer_id }
        } else {
            Outcome::AliveNoted { peer_id }
        };
        (outcome, self.forgotten(peer_id))
    }

    #[must_use]
    pub fn holds_quorum(&self, members: &Members<P>, quorum_policy: QuorumPolicy) -> bool {
        let alive = members
            .as_slice()
            .iter()
            .filter(|member| !self.is_suspected(**member))
            .count();
        quorum_policy.is_satisfied(alive)
    }

    #[must_use]
    pub fn is_suspected(&self, peer_id: P) -> bool {
        self.suspected.contains(&peer_id)
    }

    // One more period missed: counted below the threshold, suspected at it.
    #[must_use]
    pub fn missed_period(
        &self,
        peer_id: P,
        members: &Members<P>,
        threshold: usize,
    ) -> (Outcome<P>, Self) {
        let misses = self.missed(peer_id).map_or(0, |miss| miss.misses) + 1;
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
    pub fn misses(&self) -> &[MissCount<P>] {
        &self.misses
    }

    // Whom it suspects, in the order suspected, and the periods missed, in
    // the order of `members`, among `members` alone.
    #[must_use]
    pub fn of_members(&self, members: &Members<P>) -> Self {
        Self {
            suspected: members.retained(&self.suspected),
            misses: members
                .as_slice()
                .iter()
                .filter_map(|member| self.missed(*member))
                .collect(),
        }
    }

    #[must_use]
    pub fn suspected(&self) -> &[P] {
        &self.suspected
    }

    // The missed periods with `peer_id`'s now `misses`, in the members' order.
    fn counted(&self, peer_id: P, misses: usize, members: &Members<P>) -> Vec<MissCount<P>> {
        members
            .as_slice()
            .iter()
            .filter_map(|member| {
                if *member == peer_id {
                    Some(MissCount { peer_id, misses })
                } else {
                    self.missed(*member)
                }
            })
            .collect()
    }

    fn forgotten(&self, peer_id: P) -> Self {
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

    // The periods `peer_id` has missed, if it has missed any.
    fn missed(&self, peer_id: P) -> Option<MissCount<P>> {
        self.misses
            .iter()
            .copied()
            .find(|miss| miss.peer_id == peer_id)
    }
}

impl<P: PeerId> Default for Suspicion<P> {
    fn default() -> Self {
        Self {
            suspected: Vec::new(),
            misses: Vec::new(),
        }
    }
}
