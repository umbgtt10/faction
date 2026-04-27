// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cell::Cell;

use crate::machine_config::MachineConfig;
use crate::machine_input::MachineInput;
use crate::machine_observer::MachineObserver;
use crate::machine_output::MachineOutput;
use crate::machine_snapshot::MachineSnapshot;
use crate::machine_state::MachineState;
use crate::machine_transition::MachineTransition;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::states::initial::Initial;

pub struct Machine {
    config: MachineConfig,
    observer: Box<dyn MachineObserver>,
    state: Option<Box<dyn MachineState>>,
    cached_snapshot: Cell<Option<MachineSnapshot>>,
}

impl Machine {
    #[must_use]
    pub fn new(config: MachineConfig, observer: Box<dyn MachineObserver>) -> Self {
        Self {
            config,
            observer,
            state: Some(Box::new(Initial)),
            cached_snapshot: Cell::new(None),
        }
    }

    fn base_snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::new(
            ReadinessLifecycleState::Phase1Active,
            None,
            false,
            false,
            0,
            0,
            self.config.quorum_threshold(),
        )
    }

    fn compute_snapshot(&self) -> MachineSnapshot {
        let base = self.base_snapshot();
        let snap = self.state.as_ref().unwrap().state_snapshot(&base);
        self.cached_snapshot.set(Some(snap));
        snap
    }

    #[must_use]
    pub fn apply(&mut self, input: MachineInput) -> Vec<MachineOutput> {
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

        let transition = MachineTransition::new(previous_snapshot, outputs.clone(), new_snapshot);
        self.observer.observe(input, transition);
        outputs
    }

    #[must_use]
    pub fn snapshot(&self) -> MachineSnapshot {
        match self.cached_snapshot.get() {
            Some(snap) => snap,
            None => self.compute_snapshot(),
        }
    }

    #[must_use]
    pub fn config(&self) -> &MachineConfig {
        &self.config
    }
}
