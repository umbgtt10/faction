// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec::Vec;

use crate::cluster_readiness_output::ClusterReadinessOutput;
use crate::cluster_readiness_snapshot::ClusterReadinessSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterReadinessTransition {
    previous_state: ClusterReadinessSnapshot,
    outputs: Vec<ClusterReadinessOutput>,
    new_state: ClusterReadinessSnapshot,
}

impl ClusterReadinessTransition {
    #[must_use]
    pub fn new(
        previous_state: ClusterReadinessSnapshot,
        outputs: Vec<ClusterReadinessOutput>,
        new_state: ClusterReadinessSnapshot,
    ) -> Self {
        Self {
            previous_state,
            outputs,
            new_state,
        }
    }

    #[must_use]
    pub const fn previous_state(&self) -> ClusterReadinessSnapshot {
        self.previous_state
    }

    #[must_use]
    pub fn outputs(&self) -> &[ClusterReadinessOutput] {
        &self.outputs
    }

    #[must_use]
    pub const fn new_state(&self) -> ClusterReadinessSnapshot {
        self.new_state
    }
}
