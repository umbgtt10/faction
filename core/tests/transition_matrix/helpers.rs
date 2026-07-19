// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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

pub fn all_admissible() -> Vec<Command> {
    vec![
        Command::ParticipationObserved { peer_id: 0 },
        Command::ReadyObserved { peer_id: 0 },
        Command::LocalParticipationCompleted,
        Command::DeadlineExpired,
        Command::Probe,
    ]
}

pub fn collecting_admissible() -> Vec<Command> {
    vec![
        Command::ReadyObserved { peer_id: 0 },
        Command::DeadlineExpired,
        Command::Probe,
    ]
}

pub fn probe_only() -> Vec<Command> {
    vec![Command::Probe]
}

pub fn bootstrapped_admissible() -> Vec<Command> {
    vec![
        Command::ParticipationObserved { peer_id: 0 },
        Command::Probe,
    ]
}
