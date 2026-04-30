// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::state_snapshot::StateClusterView;

pub trait State: StateClusterView {
    fn step(&self, input: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>);

    fn accept(&self, _input: &Command) -> bool {
        true
    }

    fn admissible_commands(&self) -> Vec<Command> {
        vec![
            Command::ParticipationObserved {
                peer_id: 0,
                freshness: 0,
                current_marker: 0,
            },
            Command::ReadyObserved {
                peer_id: 0,
                freshness: 0,
                current_marker: 0,
            },
            Command::LocalParticipationCompleted,
            Command::DeadlineExpired,
            Command::Probe,
        ]
    }
}
