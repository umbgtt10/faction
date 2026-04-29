// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec::Vec;

use crate::outcome::Outcome;
use crate::snapshot::Snapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    previous_state: Snapshot,
    outputs: Vec<Outcome>,
    new_state: Snapshot,
}

impl Transition {
    #[must_use]
    pub fn new(previous_state: Snapshot, outputs: Vec<Outcome>, new_state: Snapshot) -> Self {
        Self {
            previous_state,
            outputs,
            new_state,
        }
    }

    #[must_use]
    pub const fn previous_state(&self) -> Snapshot {
        self.previous_state
    }

    #[must_use]
    pub fn outputs(&self) -> &[Outcome] {
        &self.outputs
    }

    #[must_use]
    pub const fn new_state(&self) -> Snapshot {
        self.new_state
    }
}
