// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::instance::peer_id::PeerId;
use crate::members::Members;
use crate::quorum_policy::QuorumPolicy;
use crate::types::Epoch;

// A decision on who may be counted, or how many are required: a join decision,
// a committed addition or removal, a new required count, or a membership
// handed to the node, adopted whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MembershipDecision<P: PeerId = u64> {
    Requested(P),
    Approved(P),
    Rejected(P),
    Committed(P),
    Removed(P),
    Counted(QuorumPolicy),
    Adopted(Members<P>, QuorumPolicy, Epoch),
}

impl<P: PeerId> MembershipDecision<P> {
    #[must_use]
    pub fn of(command: &Command<P>) -> Option<Self> {
        match command {
            Command::JoinRequested { peer_id } => Some(Self::Requested(*peer_id)),
            Command::JoinApproved { peer_id } => Some(Self::Approved(*peer_id)),
            Command::JoinRejected { peer_id } => Some(Self::Rejected(*peer_id)),
            Command::AdditionCommitted { peer_id, .. } => Some(Self::Committed(*peer_id)),
            Command::RemovalCommitted { peer_id, .. } => Some(Self::Removed(*peer_id)),
            Command::RequiredCountChanged { required_count, .. } => {
                Some(Self::Counted(QuorumPolicy::new(*required_count)))
            }
            Command::MembershipReceived {
                members,
                required_count,
                epoch,
            } => Some(Self::Adopted(
                Members::new(members.clone()),
                QuorumPolicy::new(*required_count),
                *epoch,
            )),
            _ => None,
        }
    }
}
