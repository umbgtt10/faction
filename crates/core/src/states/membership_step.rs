// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::members::Members;
use crate::outcome::Outcome;
use crate::quorum_policy::QuorumPolicy;
use crate::states::membership_decision::MembershipDecision;

// A membership decision applied, alike in every stage that admits it: a
// committed addition admits its peer as an approval does, and a new count
// becomes the count in effect.
pub struct MembershipStep {
    outcomes: Vec<Outcome>,
    members: Members,
    quorum_policy: QuorumPolicy,
}

impl MembershipStep {
    #[must_use]
    pub fn new(
        members: Members,
        quorum_policy: QuorumPolicy,
        decision: MembershipDecision,
    ) -> Self {
        match decision {
            MembershipDecision::Requested(peer_id) => Self {
                outcomes: vec![Outcome::EmitJoinRequest { peer_id }],
                members,
                quorum_policy,
            },
            MembershipDecision::Approved(peer_id) | MembershipDecision::Committed(peer_id) => {
                match members.with_admitted(peer_id) {
                    Some(admitted) => Self {
                        outcomes: vec![Outcome::MemberAdmitted { peer_id }],
                        members: admitted,
                        quorum_policy,
                    },
                    None => Self {
                        outcomes: vec![Outcome::DuplicateMemberIgnored { peer_id }],
                        members,
                        quorum_policy,
                    },
                }
            }
            MembershipDecision::Rejected(peer_id) => Self {
                outcomes: vec![Outcome::JoinDenied { peer_id }],
                members,
                quorum_policy,
            },
            MembershipDecision::Counted(counted) => Self {
                outcomes: vec![Outcome::RequiredCountSet {
                    required_count: counted.threshold(),
                }],
                members,
                quorum_policy: counted,
            },
        }
    }

    #[must_use]
    pub fn members(&self) -> &Members {
        &self.members
    }

    #[must_use]
    pub fn outcomes(&self) -> &[Outcome] {
        &self.outcomes
    }

    #[must_use]
    pub fn quorum_policy(&self) -> QuorumPolicy {
        self.quorum_policy
    }
}
