// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::instance::peer_id::PeerId;
use alloc::vec;
use alloc::vec::Vec;

use crate::members::Members;
use crate::outcome::Outcome;
use crate::quorum_policy::QuorumPolicy;
use crate::states::membership_decision::MembershipDecision;

// A membership decision applied, alike in every stage that admits it: a
// committed addition admits its peer as an approval does, a committed removal
// takes its peer out, a new count becomes the count in effect, and a membership
// adopted replaces both.
pub struct MembershipStep<P: PeerId = u64> {
    outcomes: Vec<Outcome<P>>,
    members: Members<P>,
    quorum_policy: QuorumPolicy,
}

impl<P: PeerId> MembershipStep<P> {
    #[must_use]
    pub fn new(
        members: Members<P>,
        quorum_policy: QuorumPolicy,
        decision: MembershipDecision<P>,
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
            MembershipDecision::Removed(peer_id) => match members.without(peer_id) {
                Some(rest) => Self {
                    outcomes: vec![Outcome::MemberRemoved { peer_id }],
                    members: rest,
                    quorum_policy,
                },
                None => Self {
                    outcomes: vec![Outcome::NonMemberIgnored { peer_id }],
                    members,
                    quorum_policy,
                },
            },
            MembershipDecision::Counted(counted) => Self {
                outcomes: vec![Outcome::RequiredCountSet {
                    required_count: counted.threshold(),
                }],
                members,
                quorum_policy: counted,
            },
            MembershipDecision::Adopted(adopted, quorum_policy, epoch) => Self {
                outcomes: vec![Outcome::MembershipAdopted { epoch }],
                members: adopted,
                quorum_policy,
            },
        }
    }

    #[must_use]
    pub fn members(&self) -> &Members<P> {
        &self.members
    }

    #[must_use]
    pub fn outcomes(&self) -> &[Outcome<P>] {
        &self.outcomes
    }

    #[must_use]
    pub fn quorum_policy(&self) -> QuorumPolicy {
        self.quorum_policy
    }
}
