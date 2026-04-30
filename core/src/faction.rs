// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;

use crate::command::Command;
use crate::config::Config;
use crate::observer::Observer;
use crate::process_result::ProcessResult;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::snapshot::Snapshot;
use crate::state::State;
use crate::states::initial::Initial;
use crate::transition::Transition;

pub struct Faction {
    config: Config,
    observer: Box<dyn Observer>,
    state: Box<dyn State>,
    cached_snapshot: Snapshot,
}

impl Faction {
    #[must_use]
    pub fn new(config: Config, observer: Box<dyn Observer>) -> Self {
        let state: Box<dyn State> = Box::new(Initial);
        let base = Snapshot::new(
            ReadinessLifecycleState::Phase1Active,
            None,
            false,
            false,
            0,
            0,
            config.quorum_threshold(),
        );
        let snapshot = state.state_snapshot(&base);
        Self {
            config,
            observer,
            state,
            cached_snapshot: snapshot,
        }
    }

    #[must_use]
    pub fn process(&mut self, command: Command) -> ProcessResult {
        if let Command::Probe = command {
            return ProcessResult::Probed {
                snapshot: self.cached_snapshot,
                admissible: self.state.admissible_commands(),
            };
        }

        if !self.state.accept(&command) {
            return ProcessResult::Rejected {
                snapshot: self.cached_snapshot,
                admissible: self.state.admissible_commands(),
            };
        }

        let previous_snapshot = self.cached_snapshot;

        let (outputs, new_state) = self.state.step(command, &self.config);
        self.state = new_state;

        let new_snapshot = self.state.state_snapshot(&previous_snapshot);
        self.cached_snapshot = new_snapshot;

        let transition = Transition::new(previous_snapshot, outputs.clone(), new_snapshot);
        self.observer.observe(command, transition);
        ProcessResult::Accepted {
            outcomes: outputs,
            snapshot: new_snapshot,
        }
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
}
