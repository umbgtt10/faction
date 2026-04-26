// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

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
    state: Box<dyn VibeState>,
}

impl Vibe {
    #[must_use]
    pub fn new(config: VibeConfig, observer: Box<dyn VibeObserver>) -> Self {
        Self {
            config,
            observer,
            state: Box::new(Initial),
        }
    }

    #[must_use]
    pub fn apply(&mut self, input: VibeInput) -> Vec<VibeOutput> {
        if !self.state.deal(&input) {
            return Vec::new();
        }

        let previous_snapshot = self.snapshot();
        let old_state = core::mem::replace(&mut self.state, Box::new(Initial));
        let (outputs, new_state) = old_state.punch(input, &self.config);
        self.state = new_state;
        let new_snapshot = self.snapshot();
        let transition = VibeTransition::new(previous_snapshot, outputs.clone(), new_snapshot);
        self.observer.observe(input, transition);
        outputs
    }

    #[must_use]
    pub fn snapshot(&self) -> VibeSnapshot {
        self.state.vibe_check(self.config.quorum_threshold())
    }

    #[must_use]
    pub fn config(&self) -> &VibeConfig {
        &self.config
    }
}
