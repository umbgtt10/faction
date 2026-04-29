// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::outcome::Outcome;
use faction::snapshot::Snapshot;
use faction::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioTraceEntry {
    node_id: PeerId,
    input: Command,
    outputs: alloc::vec::Vec<Outcome>,
    snapshot: Snapshot,
}

impl ScenarioTraceEntry {
    #[must_use]
    pub fn new(
        node_id: PeerId,
        input: Command,
        outputs: alloc::vec::Vec<Outcome>,
        snapshot: Snapshot,
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
    pub const fn input(&self) -> Command {
        self.input
    }

    #[must_use]
    pub fn outputs(&self) -> &[Outcome] {
        &self.outputs
    }

    #[must_use]
    pub const fn snapshot(&self) -> Snapshot {
        self.snapshot
    }
}
