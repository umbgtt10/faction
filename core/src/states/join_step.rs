// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::command::Command;
use crate::members::Members;
use crate::outcome::Outcome;

pub struct JoinStep {
    outcomes: Vec<Outcome>,
    members: Members,
}

impl JoinStep {
    #[must_use]
    pub fn new(members: Members, command: &Command) -> Self {
        match command {
            Command::JoinRequested { peer_id } => Self {
                outcomes: vec![Outcome::EmitJoinRequest { peer_id: *peer_id }],
                members,
            },
            Command::JoinApproved { peer_id } => {
                if members.is_member(*peer_id) {
                    Self {
                        outcomes: vec![Outcome::DuplicateMemberIgnored { peer_id: *peer_id }],
                        members,
                    }
                } else {
                    Self {
                        outcomes: vec![Outcome::MemberAdmitted { peer_id: *peer_id }],
                        members: members.with_admitted(*peer_id),
                    }
                }
            }
            Command::JoinRejected { peer_id } => Self {
                outcomes: vec![Outcome::JoinDenied { peer_id: *peer_id }],
                members,
            },
            _ => unreachable!("JoinStep only handles join commands"),
        }
    }

    #[must_use]
    pub fn outcomes(&self) -> &[Outcome] {
        &self.outcomes
    }

    #[must_use]
    pub fn members(&self) -> &Members {
        &self.members
    }
}
