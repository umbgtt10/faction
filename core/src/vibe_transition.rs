// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec::Vec;

use crate::vibe_output::VibeOutput;
use crate::vibe_snapshot::VibeSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VibeTransition {
    previous_state: VibeSnapshot,
    outputs: Vec<VibeOutput>,
    new_state: VibeSnapshot,
}

impl VibeTransition {
    #[must_use]
    pub fn new(
        previous_state: VibeSnapshot,
        outputs: Vec<VibeOutput>,
        new_state: VibeSnapshot,
    ) -> Self {
        Self {
            previous_state,
            outputs,
            new_state,
        }
    }

    #[must_use]
    pub const fn previous_state(&self) -> VibeSnapshot {
        self.previous_state
    }

    #[must_use]
    pub fn outputs(&self) -> &[VibeOutput] {
        &self.outputs
    }

    #[must_use]
    pub const fn new_state(&self) -> VibeSnapshot {
        self.new_state
    }
}
