// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cell::Cell;

use crate::states::initial::Initial;
use crate::vibe_config::VibeConfig;
use crate::vibe_input::VibeInput;
use crate::vibe_observer::VibeObserver;
use crate::vibe_output::VibeOutput;
use crate::vibe_snapshot::VibeSnapshot;
use crate::vibe_state::VibeState;
use crate::vibe_transition::VibeTransition;

pub struct Vibe {
    config: VibeConfig,
    observer: Box<dyn VibeObserver>,
    state: Option<Box<dyn VibeState>>,
    cached_snapshot: Cell<Option<VibeSnapshot>>,
}

impl Vibe {
    #[must_use]
    pub fn new(config: VibeConfig, observer: Box<dyn VibeObserver>) -> Self {
        Self {
            config,
            observer,
            state: Some(Box::new(Initial)),
            cached_snapshot: Cell::new(None),
        }
    }

    #[must_use]
    pub fn apply(&mut self, input: VibeInput) -> Vec<VibeOutput> {
        if !self.state.as_ref().unwrap().deal(&input) {
            return Vec::new();
        }

        let previous_snapshot = match self.cached_snapshot.get() {
            Some(snap) => snap,
            None => {
                let snap = self
                    .state
                    .as_ref()
                    .unwrap()
                    .vibe_check(self.config.quorum_threshold());
                self.cached_snapshot.set(Some(snap));
                snap
            }
        };

        let old_state = self.state.take().unwrap();
        let (outputs, new_state) = old_state.punch(input, &self.config);
        self.state = Some(new_state);

        let new_snapshot = self
            .state
            .as_ref()
            .unwrap()
            .vibe_check(self.config.quorum_threshold());
        self.cached_snapshot.set(Some(new_snapshot));

        let transition = VibeTransition::new(previous_snapshot, outputs.clone(), new_snapshot);
        self.observer.observe(input, transition);
        outputs
    }

    #[must_use]
    pub fn snapshot(&self) -> VibeSnapshot {
        match self.cached_snapshot.get() {
            Some(snap) => snap,
            None => {
                let snap = self
                    .state
                    .as_ref()
                    .unwrap()
                    .vibe_check(self.config.quorum_threshold());
                self.cached_snapshot.set(Some(snap));
                snap
            }
        }
    }

    #[must_use]
    pub fn config(&self) -> &VibeConfig {
        &self.config
    }
}
