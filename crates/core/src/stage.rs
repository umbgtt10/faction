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
use crate::states::reconfiguring::Reconfiguring;

const COMMANDS: [Command; 17] = [
    Command::ParticipationObserved { peer_id: 0 },
    Command::ReadyObserved { peer_id: 0 },
    Command::LocalParticipationCompleted,
    Command::DeadlineExpired,
    Command::JoinRequested { peer_id: 0 },
    Command::JoinApproved { peer_id: 0 },
    Command::JoinRejected { peer_id: 0 },
    Command::AdditionRequested { peer_id: 0 },
    Command::MembershipAcknowledged { peer_id: 0 },
    Command::AdditionCommitted { peer_id: 0 },
    Command::AdditionAborted { peer_id: 0 },
    Command::RequiredCountChanged { required_count: 0 },
    Command::ProbeDue,
    Command::PeerHeard { peer_id: 0 },
    Command::ProbeMissed { peer_id: 0 },
    Command::IndirectProbesMissed { peer_id: 0 },
    Command::Probe,
];

pub enum Stage {
    Initial(Initial),
    Pinging(Pinging),
    Collecting(Collecting),
    Bootstrapped(Bootstrapped),
    Reconfiguring(Reconfiguring),
}

impl Stage {
    #[must_use]
    pub fn admissible_commands(&self, config: &Config) -> Vec<Command> {
        COMMANDS
            .into_iter()
            .filter(|command| *command == Command::Probe || self.step(*command, config).is_some())
            .collect()
    }

    fn current(&self) -> &dyn State {
        match self {
            Self::Initial(initial) => initial,
            Self::Pinging(pinging) => pinging,
            Self::Collecting(collecting) => collecting,
            Self::Bootstrapped(bootstrapped) => bootstrapped,
            Self::Reconfiguring(reconfiguring) => reconfiguring,
        }
    }
}

impl State for Stage {
    fn step(&self, command: Command, config: &Config) -> Option<(Vec<Outcome>, Stage)> {
        self.current().step(command, config)
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        self.current().cluster_view(previous)
    }
}
