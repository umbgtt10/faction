// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::state::State;
use crate::states::pinging::Pinging;

pub struct Initial;

impl State for Initial {
    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        previous
            .with_lifecycle_state(ReadinessLifecycleState::Phase1Active)
            .with_phase1_count(0)
            .with_phase2_count(0)
    }

    fn step(&self, input: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        let pinging = Pinging::new(config.peer_count());
        pinging.step(input, config)
    }

    fn accept(&self, input: &Command) -> bool {
        matches!(
            input,
            Command::ParticipationObserved { .. } | Command::ReadyObserved { .. }
        )
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
            Command::Probe,
        ]
    }
}
