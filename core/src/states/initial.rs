// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::state::State;
use crate::states::pinging::Pinging;

#[derive(Default)]
pub struct Initial;

impl Initial {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl State for Initial {
    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        previous
            .clone()
            .with_peer_state(PeerState::Fresh)
            .with_is_pinging_completed(false)
            .with_pinging_peers(Vec::new())
            .with_collecting_peers(Vec::new())
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        let pinging = Pinging::new();
        pinging.step(command, config)
    }

    fn accept(&self, command: &Command) -> bool {
        matches!(
            command,
            Command::ParticipationObserved { .. }
                | Command::ReadyObserved { .. }
                | Command::LocalParticipationCompleted
        )
    }

    fn admissible_commands(&self) -> Vec<Command> {
        vec![
            Command::ParticipationObserved { peer_id: 0 },
            Command::ReadyObserved { peer_id: 0 },
            Command::LocalParticipationCompleted,
            Command::Probe,
        ]
    }
}
