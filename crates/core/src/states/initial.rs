// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::config::Config;
use crate::members::Members;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::stage::Stage;
use crate::state::State;
use crate::states::join_decision::JoinDecision;
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

    fn step(&self, command: Command, config: &Config) -> Option<(Vec<Outcome>, Stage)> {
        if let Some(decision) = JoinDecision::of(&command) {
            let join = JoinStep::new(self.members.clone(), decision);
            Some((
                join.outcomes().to_vec(),
                Stage::Initial(Self::new(join.members().clone())),
            ))
        } else {
            match command {
                Command::DeadlineExpired => None,
                _ => Pinging::new(self.members.clone()).step(command, config),
            }
        }
    }
}
