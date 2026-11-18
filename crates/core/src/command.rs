// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::command_kind::CommandKind;
use crate::types::Epoch;
use crate::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    ParticipationObserved {
        peer_id: PeerId,
        epoch: Epoch,
    },
    ReadyObserved {
        peer_id: PeerId,
        epoch: Epoch,
    },
    LocalParticipationCompleted,
    DeadlineExpired,
    JoinRequested {
        peer_id: PeerId,
    },
    JoinApproved {
        peer_id: PeerId,
    },
    JoinRejected {
        peer_id: PeerId,
    },
    AdditionRequested {
        peer_id: PeerId,
    },
    MembershipAcknowledged {
        peer_id: PeerId,
        epoch: Epoch,
    },
    AdditionCommitted {
        peer_id: PeerId,
        epoch: Epoch,
    },
    AdditionAborted {
        peer_id: PeerId,
    },
    RemovalRequested {
        peer_id: PeerId,
    },
    RemovalCommitted {
        peer_id: PeerId,
        epoch: Epoch,
    },
    RemovalAborted {
        peer_id: PeerId,
    },
    ProposalDeclined {
        peer_id: PeerId,
        epoch: Epoch,
    },
    RequiredCountChanged {
        required_count: usize,
        epoch: Epoch,
    },
    MembershipReceived {
        members: Vec<PeerId>,
        required_count: usize,
        epoch: Epoch,
    },
    ProbeDue,
    PeerHeard {
        peer_id: PeerId,
    },
    ProbeMissed {
        peer_id: PeerId,
    },
    IndirectProbesMissed {
        peer_id: PeerId,
    },
    Probe,
}

impl Command {
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
