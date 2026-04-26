// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::vibe_input::VibeInput;
use faction::vibe_output::VibeOutput;
use faction::vibe_snapshot::VibeSnapshot;
use faction::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VibeScenarioTraceEntry {
    node_id: PeerId,
    input: VibeInput,
    outputs: alloc::vec::Vec<VibeOutput>,
    snapshot: VibeSnapshot,
}

impl VibeScenarioTraceEntry {
    #[must_use]
    pub fn new(
        node_id: PeerId,
        input: VibeInput,
        outputs: alloc::vec::Vec<VibeOutput>,
        snapshot: VibeSnapshot,
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
    pub const fn input(&self) -> VibeInput {
        self.input
    }

    #[must_use]
    pub fn outputs(&self) -> &[VibeOutput] {
        &self.outputs
    }

    #[must_use]
    pub const fn snapshot(&self) -> VibeSnapshot {
        self.snapshot
    }
}
