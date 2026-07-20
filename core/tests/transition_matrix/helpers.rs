// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use faction::command::Command;
use faction::types::PeerId;

pub fn participation(peer_id: PeerId) -> Command {
    Command::ParticipationObserved { peer_id }
}

pub fn ready(peer_id: PeerId) -> Command {
    Command::ReadyObserved { peer_id }
}

pub fn join_requested(peer_id: PeerId) -> Command {
    Command::JoinRequested { peer_id }
}

pub fn join_approved(peer_id: PeerId) -> Command {
    Command::JoinApproved { peer_id }
}

pub fn join_rejected(peer_id: PeerId) -> Command {
    Command::JoinRejected { peer_id }
}

pub fn initial_admissible() -> Vec<Command> {
    vec![
        Command::ParticipationObserved { peer_id: 0 },
        Command::ReadyObserved { peer_id: 0 },
        Command::LocalParticipationCompleted,
        Command::JoinRequested { peer_id: 0 },
        Command::JoinApproved { peer_id: 0 },
        Command::JoinRejected { peer_id: 0 },
        Command::Probe,
    ]
}

pub fn all_admissible() -> Vec<Command> {
    vec![
        Command::ParticipationObserved { peer_id: 0 },
        Command::ReadyObserved { peer_id: 0 },
        Command::LocalParticipationCompleted,
        Command::DeadlineExpired,
        Command::JoinRequested { peer_id: 0 },
        Command::JoinApproved { peer_id: 0 },
        Command::JoinRejected { peer_id: 0 },
        Command::Probe,
    ]
}

pub fn collecting_admissible() -> Vec<Command> {
    vec![
        Command::ReadyObserved { peer_id: 0 },
        Command::DeadlineExpired,
        Command::JoinRequested { peer_id: 0 },
        Command::JoinApproved { peer_id: 0 },
        Command::JoinRejected { peer_id: 0 },
        Command::Probe,
    ]
}

pub fn probe_only() -> Vec<Command> {
    vec![Command::Probe]
}

pub fn bootstrapped_admissible() -> Vec<Command> {
    vec![
        Command::ParticipationObserved { peer_id: 0 },
        Command::JoinRequested { peer_id: 0 },
        Command::JoinApproved { peer_id: 0 },
        Command::JoinRejected { peer_id: 0 },
        Command::Probe,
    ]
}
