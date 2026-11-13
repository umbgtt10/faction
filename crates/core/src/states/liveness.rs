// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;
use core::iter::once;

use crate::command::Command;
use crate::config::Config;
use crate::members::Members;
use crate::miss_count::MissCount;
use crate::outcome::Outcome;
use crate::quorum_policy::QuorumPolicy;
use crate::states::probing::Probing;
use crate::states::suspicion::Suspicion;
use crate::types::PeerId;

// What a concluded node keeps of its members' liveness -- whom it probes and
// whom it suspects -- answering the four commands of liveness, and saying when
// a change crosses the quorum.
#[derive(Debug, Clone, Default)]
pub struct Liveness {
    probing: Probing,
    suspicion: Suspicion,
}

impl Liveness {
    // The outcome that says a change crossed the quorum, if it did.
    #[must_use]
    pub fn crossing(held: bool, holds: bool) -> Option<Outcome> {
        match (held, holds) {
            (true, false) => Some(Outcome::QuorumLost),
            (false, true) => Some(Outcome::QuorumRestored),
            _ => None,
        }
    }

    #[must_use]
    pub fn holds_quorum(&self, members: &Members, quorum_policy: QuorumPolicy) -> bool {
        self.suspicion.holds_quorum(members, quorum_policy)
    }

    #[must_use]
    pub fn last_probed(&self) -> Option<PeerId> {
        self.probing.last_probed()
    }

    #[must_use]
    pub fn misses(&self) -> &[MissCount] {
        self.suspicion.misses()
    }

    // The liveness of `members` alone: a peer no longer a member is neither
    // suspected nor counted, nor the member probed last.
    #[must_use]
    pub fn of_members(&self, members: &Members) -> Self {
        Self {
            probing: self.probing.of_members(members),
            suspicion: self.suspicion.of_members(members),
        }
    }

    #[must_use]
    pub fn step(
        &self,
        command: Command,
        members: &Members,
        quorum_policy: QuorumPolicy,
        config: &Config,
    ) -> Option<(Vec<Outcome>, Self)> {
        match command {
            Command::ProbeDue => {
                let (outcomes, probing) = self.probing.probe(members, config.peer_id());
                Some((
                    outcomes,
                    Self {
                        probing,
                        ..self.clone()
                    },
                ))
            }
            Command::PeerHeard { peer_id }
            | Command::ProbeMissed { peer_id }
            | Command::IndirectProbesMissed { peer_id } => {
                Some(self.observed(command, peer_id, members, quorum_policy, config))
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn suspected(&self) -> &[PeerId] {
        self.suspicion.suspected()
    }

    // A changed suspicion, with the quorum it crosses said last.
    fn changed(
        &self,
        (outcome, suspicion): (Outcome, Suspicion),
        members: &Members,
        quorum_policy: QuorumPolicy,
    ) -> (Vec<Outcome>, Self) {
        let crossing = Self::crossing(
            self.suspicion.holds_quorum(members, quorum_policy),
            suspicion.holds_quorum(members, quorum_policy),
        );
        (
            once(outcome).chain(crossing).collect(),
            Self {
                suspicion,
                ..self.clone()
            },
        )
    }

    // A member heard from, missed, or missed through others: the node itself
    // and a non-member are ignored, and a suspect is not suspected twice.
    fn observed(
        &self,
        command: Command,
        peer_id: PeerId,
        members: &Members,
        quorum_policy: QuorumPolicy,
        config: &Config,
    ) -> (Vec<Outcome>, Self) {
        match command {
            _ if peer_id == config.peer_id() => {
                (vec![Outcome::SelfIgnored { peer_id }], self.clone())
            }
            _ if !members.is_member(peer_id) => {
                (vec![Outcome::NonMemberIgnored { peer_id }], self.clone())
            }
            Command::PeerHeard { .. } => {
                self.changed(self.suspicion.heard(peer_id), members, quorum_policy)
            }
            _ if self.suspicion.is_suspected(peer_id) => {
                (vec![Outcome::AlreadySuspected { peer_id }], self.clone())
            }
            Command::ProbeMissed { .. } => (
                vec![Outcome::SendIndirectProbes {
                    peer_id,
                    via: Probing::helpers(
                        members,
                        peer_id,
                        config.peer_id(),
                        self.suspicion.suspected(),
                    ),
                }],
                self.clone(),
            ),
            _ => self.changed(
                self.suspicion
                    .missed_period(peer_id, members, config.suspicion_threshold()),
                members,
                quorum_policy,
            ),
        }
    }
}
