// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::machine_input::MachineInput;
use faction::machine_output::MachineOutput;
use faction::machine_snapshot::MachineSnapshot;
use faction::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VibeScenarioTraceEntry {
    node_id: PeerId,
    input: MachineInput,
    outputs: alloc::vec::Vec<MachineOutput>,
    snapshot: MachineSnapshot,
}

impl VibeScenarioTraceEntry {
    #[must_use]
    pub fn new(
        node_id: PeerId,
        input: MachineInput,
        outputs: alloc::vec::Vec<MachineOutput>,
        snapshot: MachineSnapshot,
    ) -> Self {
        Self {
            node_id,
            input,
            outputs,
            snapshot,
        }
    }

    #[must_use]
    pub const fn node_id(&self) -> PeerId {
        self.node_id
    }

    #[must_use]
    pub const fn input(&self) -> MachineInput {
        self.input
    }

    #[must_use]
    pub fn outputs(&self) -> &[MachineOutput] {
        &self.outputs
    }

    #[must_use]
    pub const fn snapshot(&self) -> MachineSnapshot {
        self.snapshot
    }
}
