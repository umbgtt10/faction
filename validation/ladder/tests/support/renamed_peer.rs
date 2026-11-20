// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use core::fmt;
use core::fmt::Debug;
use core::fmt::Formatter;
use faction::command::Command;
use faction::instance::peer_id::PeerId;

// Every bit of a renamed peer id is the u64's under this mask, so no renamed
// id holds the bytes of the one it renames.
const MASK: u64 = 0x5A5A_A5A5_0F0F_F0F0;

// A peer id of another type than u64: a u64 renamed one to one into eight
// bytes, which the machine can only copy and compare. It prints as the u64 it
// renames, so a node on renamed ids that answers alike prints alike.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RenamedPeer {
    bytes: [u8; 8],
}

impl RenamedPeer {
    pub fn new(peer: u64) -> Self {
        Self {
            bytes: (peer ^ MASK).to_le_bytes(),
        }
    }

    // `command` with every peer it names renamed.
    pub fn command(command: &Command) -> Command<Self> {
        match command.clone() {
            Command::ParticipationObserved { peer_id, epoch } => Command::ParticipationObserved {
                peer_id: Self::new(peer_id),
                epoch,
            },
            Command::ReadyObserved { peer_id, epoch } => Command::ReadyObserved {
                peer_id: Self::new(peer_id),
                epoch,
            },
            Command::LocalParticipationCompleted => Command::LocalParticipationCompleted,
            Command::DeadlineExpired => Command::DeadlineExpired,
            Command::JoinRequested { peer_id } => Command::JoinRequested {
                peer_id: Self::new(peer_id),
            },
            Command::JoinApproved { peer_id } => Command::JoinApproved {
                peer_id: Self::new(peer_id),
            },
            Command::JoinRejected { peer_id } => Command::JoinRejected {
                peer_id: Self::new(peer_id),
            },
            Command::AdditionRequested { peer_id } => Command::AdditionRequested {
                peer_id: Self::new(peer_id),
            },
            Command::MembershipAcknowledged { peer_id, epoch } => Command::MembershipAcknowledged {
                peer_id: Self::new(peer_id),
                epoch,
            },
            Command::AdditionCommitted { peer_id, epoch } => Command::AdditionCommitted {
                peer_id: Self::new(peer_id),
                epoch,
            },
            Command::AdditionAborted { peer_id } => Command::AdditionAborted {
                peer_id: Self::new(peer_id),
            },
            Command::RemovalRequested { peer_id, leaves } => Command::RemovalRequested {
                peer_id: Self::new(peer_id),
                leaves,
            },
            Command::RemovalCommitted { peer_id, epoch } => Command::RemovalCommitted {
                peer_id: Self::new(peer_id),
                epoch,
            },
            Command::RemovalAborted { peer_id } => Command::RemovalAborted {
                peer_id: Self::new(peer_id),
            },
            Command::ProposalDeclined { peer_id, epoch } => Command::ProposalDeclined {
                peer_id: Self::new(peer_id),
                epoch,
            },
            Command::RequiredCountChanged {
                required_count,
                epoch,
            } => Command::RequiredCountChanged {
                required_count,
                epoch,
            },
            Command::MembershipReceived {
                members,
                required_count,
                epoch,
            } => Command::MembershipReceived {
                members: members.into_iter().map(Self::new).collect(),
                required_count,
                epoch,
            },
            Command::ProbeDue => Command::ProbeDue,
            Command::PeerHeard { peer_id } => Command::PeerHeard {
                peer_id: Self::new(peer_id),
            },
            Command::ProbeMissed { peer_id } => Command::ProbeMissed {
                peer_id: Self::new(peer_id),
            },
            Command::IndirectProbesMissed { peer_id } => Command::IndirectProbesMissed {
                peer_id: Self::new(peer_id),
            },
            Command::Probe => Command::Probe,
        }
    }

    fn original(self) -> u64 {
        u64::from_le_bytes(self.bytes) ^ MASK
    }
}

impl PeerId for RenamedPeer {}

impl Debug for RenamedPeer {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.original())
    }
}
