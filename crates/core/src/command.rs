// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::command_kind::CommandKind;
use crate::instance::peer_id::PeerId;
use crate::types::Epoch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command<P: PeerId = u64> {
    ParticipationObserved {
        peer_id: P,
        epoch: Epoch,
    },
    ReadyObserved {
        peer_id: P,
        epoch: Epoch,
    },
    LocalParticipationCompleted,
    DeadlineExpired,
    JoinRequested {
        peer_id: P,
    },
    JoinApproved {
        peer_id: P,
    },
    JoinRejected {
        peer_id: P,
    },
    AdditionRequested {
        peer_id: P,
    },
    MembershipAcknowledged {
        peer_id: P,
        epoch: Epoch,
    },
    AdditionCommitted {
        peer_id: P,
        epoch: Epoch,
    },
    AdditionAborted {
        peer_id: P,
    },
    RemovalRequested {
        peer_id: P,
        leaves: Option<usize>,
    },
    RemovalCommitted {
        peer_id: P,
        epoch: Epoch,
    },
    RemovalAborted {
        peer_id: P,
    },
    ProposalDeclined {
        peer_id: P,
        epoch: Epoch,
    },
    RequiredCountChanged {
        required_count: usize,
        epoch: Epoch,
    },
    MembershipReceived {
        members: Vec<P>,
        required_count: usize,
        epoch: Epoch,
    },
    ProbeDue,
    PeerHeard {
        peer_id: P,
    },
    ProbeMissed {
        peer_id: P,
    },
    IndirectProbesMissed {
        peer_id: P,
    },
    Probe,
}

impl<P: PeerId> Command<P> {
    #[must_use]
    pub fn kind(&self) -> CommandKind {
        match self {
            Self::ParticipationObserved { .. } => CommandKind::ParticipationObserved,
            Self::ReadyObserved { .. } => CommandKind::ReadyObserved,
            Self::LocalParticipationCompleted => CommandKind::LocalParticipationCompleted,
            Self::DeadlineExpired => CommandKind::DeadlineExpired,
            Self::JoinRequested { .. } => CommandKind::JoinRequested,
            Self::JoinApproved { .. } => CommandKind::JoinApproved,
            Self::JoinRejected { .. } => CommandKind::JoinRejected,
            Self::AdditionRequested { .. } => CommandKind::AdditionRequested,
            Self::MembershipAcknowledged { .. } => CommandKind::MembershipAcknowledged,
            Self::AdditionCommitted { .. } => CommandKind::AdditionCommitted,
            Self::AdditionAborted { .. } => CommandKind::AdditionAborted,
            Self::RemovalRequested { .. } => CommandKind::RemovalRequested,
            Self::RemovalCommitted { .. } => CommandKind::RemovalCommitted,
            Self::RemovalAborted { .. } => CommandKind::RemovalAborted,
            Self::ProposalDeclined { .. } => CommandKind::ProposalDeclined,
            Self::RequiredCountChanged { .. } => CommandKind::RequiredCountChanged,
            Self::MembershipReceived { .. } => CommandKind::MembershipReceived,
            Self::ProbeDue => CommandKind::ProbeDue,
            Self::PeerHeard { .. } => CommandKind::PeerHeard,
            Self::ProbeMissed { .. } => CommandKind::ProbeMissed,
            Self::IndirectProbesMissed { .. } => CommandKind::IndirectProbesMissed,
            Self::Probe => CommandKind::Probe,
        }
    }
}
