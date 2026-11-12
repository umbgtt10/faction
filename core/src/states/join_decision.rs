// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::types::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinDecision {
    Requested(PeerId),
    Approved(PeerId),
    Rejected(PeerId),
}

impl JoinDecision {
    #[must_use]
    pub fn of(command: &Command) -> Option<Self> {
        match command {
            Command::JoinRequested { peer_id } => Some(Self::Requested(*peer_id)),
            Command::JoinApproved { peer_id } => Some(Self::Approved(*peer_id)),
            Command::JoinRejected { peer_id } => Some(Self::Rejected(*peer_id)),
            _ => None,
        }
    }
}
