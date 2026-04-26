// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::cluster_readiness_input::ClusterReadinessInput;
use faction::cluster_readiness_output::ClusterReadinessOutput;
use faction::cluster_readiness_snapshot::ClusterReadinessSnapshot;
use faction::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterReadinessScenarioTraceEntry {
    node_id: PeerId,
    input: ClusterReadinessInput,
    outputs: alloc::vec::Vec<ClusterReadinessOutput>,
    snapshot: ClusterReadinessSnapshot,
}

impl ClusterReadinessScenarioTraceEntry {
    #[must_use]
    pub fn new(
        node_id: PeerId,
        input: ClusterReadinessInput,
        outputs: alloc::vec::Vec<ClusterReadinessOutput>,
        snapshot: ClusterReadinessSnapshot,
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
    pub const fn input(&self) -> ClusterReadinessInput {
        self.input
    }

    #[must_use]
    pub fn outputs(&self) -> &[ClusterReadinessOutput] {
        &self.outputs
    }

    #[must_use]
    pub const fn snapshot(&self) -> ClusterReadinessSnapshot {
        self.snapshot
    }
}
