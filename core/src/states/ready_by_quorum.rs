// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::snapshot::Snapshot;
use crate::state::State;
use crate::state_snapshot::StateSnapshot;

pub struct ReadyByQuorum {
    pub phase1_count: usize,
    pub phase2_count: usize,
}

impl StateSnapshot for ReadyByQuorum {
    fn state_snapshot(&self, previous: &Snapshot) -> Snapshot {
        previous
            .with_lifecycle_state(ReadinessLifecycleState::ReadyByQuorum)
            .with_exit_mode(Some(ReadinessExitMode::Quorum))
            .with_local_participation_complete(true)
            .with_readiness_exited(true)
            .with_phase1_count(self.phase1_count)
            .with_phase2_count(self.phase2_count)
    }
}

impl State for ReadyByQuorum {
    fn step(self: Box<Self>, _input: Command, _config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        unreachable!("accept() rejects all inputs for this state")
    }

    fn accept(&self, _input: &Command) -> bool {
        false
    }
}
