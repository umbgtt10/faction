// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::state::State;
use crate::states::bootstrapped::Bootstrapped;
use crate::states::collecting::Collecting;
use crate::states::initial::Initial;
use crate::states::pinging::Pinging;

const COMMANDS: [Command; 8] = [
    Command::ParticipationObserved { peer_id: 0 },
    Command::ReadyObserved { peer_id: 0 },
    Command::LocalParticipationCompleted,
    Command::DeadlineExpired,
    Command::JoinRequested { peer_id: 0 },
    Command::JoinApproved { peer_id: 0 },
    Command::JoinRejected { peer_id: 0 },
    Command::Probe,
];

pub enum Stage {
    Initial(Initial),
    Pinging(Pinging),
    Collecting(Collecting),
    Bootstrapped(Bootstrapped),
}

impl Stage {
    #[must_use]
    pub fn admissible_commands(&self) -> Vec<Command> {
        COMMANDS
            .into_iter()
            .filter(|command| *command == Command::Probe || self.accept(command))
            .collect()
    }

    fn current(&self) -> &dyn State {
        match self {
            Self::Initial(initial) => initial,
            Self::Pinging(pinging) => pinging,
            Self::Collecting(collecting) => collecting,
            Self::Bootstrapped(bootstrapped) => bootstrapped,
        }
    }
}

impl State for Stage {
    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Stage) {
        self.current().step(command, config)
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        self.current().cluster_view(previous)
    }

    fn accept(&self, command: &Command) -> bool {
        self.current().accept(command)
    }
}
