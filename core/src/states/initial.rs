// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::config::Config;
use crate::members::Members;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::state::State;
use crate::states::join_step::JoinStep;
use crate::states::pinging::Pinging;

pub struct Initial {
    members: Members,
}

impl Initial {
    #[must_use]
    pub fn new(members: Members) -> Self {
        Self { members }
    }
}

impl State for Initial {
    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        ClusterViewBuilder::from_view(previous)
            .with_peer_state(PeerState::Fresh)
            .with_is_pinging_completed(false)
            .with_pinging_peers(Vec::new())
            .with_collecting_peers(Vec::new())
            .with_members(self.members.clone())
            .build()
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        match command {
            Command::JoinRequested { .. }
            | Command::JoinApproved { .. }
            | Command::JoinRejected { .. } => {
                let join = JoinStep::new(self.members.clone(), &command);
                (
                    join.outcomes().to_vec(),
                    Box::new(Self::new(join.members().clone())),
                )
            }
            _ => {
                let pinging = Pinging::new(self.members.clone());
                pinging.step(command, config)
            }
        }
    }

    fn accept(&self, command: &Command) -> bool {
        matches!(
            command,
            Command::ParticipationObserved { .. }
                | Command::ReadyObserved { .. }
                | Command::LocalParticipationCompleted
                | Command::JoinRequested { .. }
                | Command::JoinApproved { .. }
                | Command::JoinRejected { .. }
        )
    }

    fn admissible_commands(&self) -> Vec<Command> {
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
}
