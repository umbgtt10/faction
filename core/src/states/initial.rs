// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::snapshot::Snapshot;
use crate::state::State;
use crate::state_snapshot::StateSnapshot;
use crate::states::pinging::Pinging;

pub struct Initial;

impl StateSnapshot for Initial {
    fn state_snapshot(&self, previous: &Snapshot) -> Snapshot {
        previous
            .with_lifecycle_state(ReadinessLifecycleState::Phase1Active)
            .with_phase1_count(0)
            .with_phase2_count(0)
    }
}

impl State for Initial {
    fn step(self: Box<Self>, input: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        Box::new(Pinging::new(config.peer_count())).step(input, config)
    }

    fn accept(&self, input: &Command) -> bool {
        matches!(
            input,
            Command::ParticipationObserved { .. } | Command::ReadyObserved { .. }
        )
    }
}
