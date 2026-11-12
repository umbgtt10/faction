// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::members::Members;
use crate::outcome::Outcome;
use crate::states::join_decision::JoinDecision;

pub struct JoinStep {
    outcomes: Vec<Outcome>,
    members: Members,
}

impl JoinStep {
    #[must_use]
    pub fn new(members: Members, decision: JoinDecision) -> Self {
        match decision {
            JoinDecision::Requested(peer_id) => Self {
                outcomes: vec![Outcome::EmitJoinRequest { peer_id }],
                members,
            },
            JoinDecision::Approved(peer_id) => match members.with_admitted(peer_id) {
                Some(admitted) => Self {
                    outcomes: vec![Outcome::MemberAdmitted { peer_id }],
                    members: admitted,
                },
                None => Self {
                    outcomes: vec![Outcome::DuplicateMemberIgnored { peer_id }],
                    members,
                },
            },
            JoinDecision::Rejected(peer_id) => Self {
                outcomes: vec![Outcome::JoinDenied { peer_id }],
                members,
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
}
