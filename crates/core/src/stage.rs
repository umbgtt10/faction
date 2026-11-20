// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::instance::peer_id::PeerId;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::command_kind::CommandKind;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::state::State;
use crate::states::bootstrapped::Bootstrapped;
use crate::states::collecting::Collecting;
use crate::states::coordinating::Coordinating;
use crate::states::epoch_rule::EpochRule;
use crate::states::initial::Initial;
use crate::states::pinging::Pinging;
use crate::states::reconfiguring::Reconfiguring;
use crate::states::removed::Removed;
use crate::types::Epoch;

#[derive(Clone)]
pub enum Stage<P: PeerId = u64> {
    Initial(Initial<P>),
    Pinging(Pinging<P>),
    Collecting(Collecting<P>),
    Bootstrapped(Bootstrapped<P>),
    Coordinating(Coordinating<P>),
    Reconfiguring(Reconfiguring<P>),
    Removed(Removed<P>),
}

impl<P: PeerId> Stage<P> {
    #[must_use]
    pub fn admissible_commands(&self, config: &Config<P>, epoch: Epoch) -> Vec<CommandKind> {
        Self::templates(config.peer_id())
            .into_iter()
            .filter(|command| {
                command.kind() == CommandKind::Probe
                    || self.step(command.clone(), config, epoch).is_some()
            })
            .map(|command| command.kind())
            .collect()
    }

    fn current(&self) -> &dyn State<P> {
        match self {
            Self::Initial(initial) => initial,
            Self::Pinging(pinging) => pinging,
            Self::Collecting(collecting) => collecting,
            Self::Bootstrapped(bootstrapped) => bootstrapped,
            Self::Coordinating(coordinating) => coordinating,
            Self::Reconfiguring(reconfiguring) => reconfiguring,
            Self::Removed(removed) => removed,
        }
    }

    // One command of every kind, the node's own peer id in every one that
    // names a peer: what `admissible_commands` asks each stage to step on.
    fn templates(peer_id: P) -> [Command<P>; 22] {
        [
            Command::ParticipationObserved { peer_id, epoch: 0 },
            Command::ReadyObserved { peer_id, epoch: 0 },
            Command::LocalParticipationCompleted,
            Command::DeadlineExpired,
            Command::JoinRequested { peer_id },
            Command::JoinApproved { peer_id },
            Command::JoinRejected { peer_id },
            Command::AdditionRequested { peer_id },
            Command::MembershipAcknowledged { peer_id, epoch: 0 },
            Command::AdditionCommitted { peer_id, epoch: 0 },
            Command::AdditionAborted { peer_id },
            Command::RemovalRequested {
                peer_id,
                leaves: None,
            },
            Command::RemovalCommitted { peer_id, epoch: 0 },
            Command::RemovalAborted { peer_id },
            Command::ProposalDeclined { peer_id, epoch: 0 },
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
            Command::PeerHeard { peer_id },
            Command::ProbeMissed { peer_id },
            Command::IndirectProbesMissed { peer_id },
            Command::Probe,
        ]
    }
}

// A command the stage admits is answered by the stage, but a commit or a
// membership the node holds already, which its epoch answers (§16).
impl<P: PeerId> State<P> for Stage<P> {
    fn step(
        &self,
        command: Command<P>,
        config: &Config<P>,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome<P>>, Stage<P>)> {
        let held = EpochRule::new(epoch).held(&command);
        let answer = self.current().step(command, config, epoch)?;
        Some(held.map_or(answer, |ignored| (vec![ignored], self.clone())))
    }

    fn cluster_view(&self, previous: &ClusterView<P>) -> ClusterView<P> {
        self.current().cluster_view(previous)
    }
}
