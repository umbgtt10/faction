// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::Cell;

use crate::command::Command;
use crate::config::Config;
use crate::observer::Observer;
use crate::outcome::Outcome;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::snapshot::Snapshot;
use crate::state::State;
use crate::states::initial::Initial;
use crate::transition::Transition;

pub struct Faction {
    config: Config,
    observer: Box<dyn Observer>,
    state: Option<Box<dyn State>>,
    cached_snapshot: Cell<Option<Snapshot>>,
}

impl Faction {
    #[must_use]
    pub fn new(config: Config, observer: Box<dyn Observer>) -> Self {
        Self {
            config,
            observer,
            state: Some(Box::new(Initial)),
            cached_snapshot: Cell::new(None),
        }
    }

    fn base_snapshot(&self) -> Snapshot {
        Snapshot::new(
            ReadinessLifecycleState::Phase1Active,
            None,
            false,
            false,
            0,
            0,
            self.config.quorum_threshold(),
        )
    }

    fn compute_snapshot(&self) -> Snapshot {
        let base = self.base_snapshot();
        let snap = self.state.as_ref().unwrap().state_snapshot(&base);
        self.cached_snapshot.set(Some(snap));
        snap
    }

    #[must_use]
    pub fn apply(&mut self, input: Command) -> Vec<Outcome> {
        if let Command::GetSnapshot = input {
            return vec![Outcome::SnapshotAvailable(self.snapshot())];
        }

        if !self.state.as_ref().unwrap().accept(&input) {
            return Vec::new();
        }

        let previous_snapshot = match self.cached_snapshot.get() {
            Some(snap) => snap,
            None => self.compute_snapshot(),
        };

        let old_state = self.state.take().unwrap();
        let (outputs, new_state) = old_state.step(input, &self.config);
        self.state = Some(new_state);

        let new_snapshot = self
            .state
            .as_ref()
            .unwrap()
            .state_snapshot(&previous_snapshot);
        self.cached_snapshot.set(Some(new_snapshot));

        let transition = Transition::new(previous_snapshot, outputs.clone(), new_snapshot);
        self.observer.observe(input, transition);
        outputs
    }

    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        match self.cached_snapshot.get() {
            Some(snap) => snap,
            None => self.compute_snapshot(),
        }
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
}
