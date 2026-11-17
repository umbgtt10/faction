// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::state::State;
use crate::states::bootstrapped::Bootstrapped;
use crate::states::collecting::Collecting;
use crate::states::epoch_rule::EpochRule;
use crate::states::initial::Initial;
use crate::states::pinging::Pinging;
use crate::states::reconfiguring::Reconfiguring;
use crate::states::removed::Removed;
use crate::types::Epoch;

const COMMANDS: [Command; 22] = [
    Command::ParticipationObserved {
        peer_id: 0,
        epoch: 0,
    },
    Command::ReadyObserved {
        peer_id: 0,
        epoch: 0,
    },
    Command::LocalParticipationCompleted,
    Command::DeadlineExpired,
    Command::JoinRequested { peer_id: 0 },
    Command::JoinApproved { peer_id: 0 },
    Command::JoinRejected { peer_id: 0 },
    Command::AdditionRequested { peer_id: 0 },
    Command::MembershipAcknowledged {
        peer_id: 0,
        epoch: 0,
    },
    Command::AdditionCommitted {
        peer_id: 0,
        epoch: 0,
    },
    Command::AdditionAborted { peer_id: 0 },
    Command::RemovalRequested { peer_id: 0 },
    Command::RemovalCommitted {
        peer_id: 0,
        epoch: 0,
    },
    Command::RemovalAborted { peer_id: 0 },
    Command::ProposalDeclined {
        peer_id: 0,
        epoch: 0,
    },
    Command::RequiredCountChanged {
        required_count: 0,
        epoch: 0,
    },
    Command::MembershipReceived {
        members: Vec::new(),
        required_count: 0,
        epoch: 0,
    },
    Command::ProbeDue,
    Command::PeerHeard { peer_id: 0 },
    Command::ProbeMissed { peer_id: 0 },
    Command::IndirectProbesMissed { peer_id: 0 },
    Command::Probe,
];

#[derive(Clone)]
pub enum Stage {
    Initial(Initial),
    Pinging(Pinging),
    Collecting(Collecting),
    Bootstrapped(Bootstrapped),
    Reconfiguring(Reconfiguring),
    Removed(Removed),
}

impl Stage {
    #[must_use]
    pub fn admissible_commands(&self, config: &Config, epoch: Epoch) -> Vec<Command> {
        COMMANDS
            .into_iter()
            .filter(|command| {
                *command == Command::Probe || self.step(command.clone(), config, epoch).is_some()
            })
            .collect()
    }

    fn current(&self) -> &dyn State {
        match self {
            Self::Initial(initial) => initial,
            Self::Pinging(pinging) => pinging,
            Self::Collecting(collecting) => collecting,
            Self::Bootstrapped(bootstrapped) => bootstrapped,
            Self::Reconfiguring(reconfiguring) => reconfiguring,
            Self::Removed(removed) => removed,
        }
    }
}

// A command the stage admits is answered by the stage, but a commit or a
// membership the node holds already, which its epoch answers (§16).
impl State for Stage {
    fn step(
        &self,
        command: Command,
        config: &Config,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome>, Stage)> {
        let held = EpochRule::new(epoch).held(&command);
        let answer = self.current().step(command, config, epoch)?;
        Some(held.map_or(answer, |ignored| (vec![ignored], self.clone())))
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        self.current().cluster_view(previous)
    }
}
