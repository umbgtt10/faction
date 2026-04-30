// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::cluster_view::ClusterView;
use crate::state::State;
use crate::state_snapshot::StateClusterView;

pub struct TimedOut {
    pub phase1_count: usize,
    pub phase2_count: usize,
}

impl StateClusterView for TimedOut {
    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        previous
            .with_lifecycle_state(ReadinessLifecycleState::TimedOut)
            .with_phase1_count(self.phase1_count)
            .with_phase2_count(self.phase2_count)
    }
}

impl State for TimedOut {
    fn step(&self, _input: Command, _config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        unreachable!("accept() rejects all inputs for this state")
    }

    fn accept(&self, _input: &Command) -> bool {
        false
    }

    fn admissible_commands(&self) -> Vec<Command> {
        vec![Command::Probe]
    }
}
