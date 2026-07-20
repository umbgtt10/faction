// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;

pub trait State: Send {
    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>);

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView;

    fn accept(&self, _command: &Command) -> bool {
        true
    }

    fn admissible_commands(&self) -> Vec<Command> {
        vec![
            Command::ParticipationObserved { peer_id: 0 },
            Command::ReadyObserved { peer_id: 0 },
            Command::LocalParticipationCompleted,
            Command::DeadlineExpired,
            Command::Probe,
        ]
    }
}
