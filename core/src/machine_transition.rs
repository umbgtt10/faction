// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec::Vec;

use crate::machine_output::MachineOutput;
use crate::machine_snapshot::MachineSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineTransition {
    previous_state: MachineSnapshot,
    outputs: Vec<MachineOutput>,
    new_state: MachineSnapshot,
}

impl MachineTransition {
    #[must_use]
    pub fn new(
        previous_state: MachineSnapshot,
        outputs: Vec<MachineOutput>,
        new_state: MachineSnapshot,
    ) -> Self {
        Self {
            previous_state,
            outputs,
            new_state,
        }
    }

    #[must_use]
    pub const fn previous_state(&self) -> MachineSnapshot {
        self.previous_state
    }

    #[must_use]
    pub fn outputs(&self) -> &[MachineOutput] {
        &self.outputs
    }

    #[must_use]
    pub const fn new_state(&self) -> MachineSnapshot {
        self.new_state
    }
}
