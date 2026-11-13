// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::quorum_policy::QuorumPolicy;
use crate::types::PeerId;

// A decision on who may be counted, or how many are required: a join decision,
// a committed addition, or a new required count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipDecision {
    Requested(PeerId),
    Approved(PeerId),
    Rejected(PeerId),
    Committed(PeerId),
    Counted(QuorumPolicy),
}

impl MembershipDecision {
    #[must_use]
    pub fn of(command: &Command) -> Option<Self> {
        match command {
            Command::JoinRequested { peer_id } => Some(Self::Requested(*peer_id)),
            Command::JoinApproved { peer_id } => Some(Self::Approved(*peer_id)),
            Command::JoinRejected { peer_id } => Some(Self::Rejected(*peer_id)),
            Command::AdditionCommitted { peer_id } => Some(Self::Committed(*peer_id)),
            Command::RequiredCountChanged { required_count } => {
                Some(Self::Counted(QuorumPolicy::new(*required_count)))
            }
            _ => None,
        }
    }
}
