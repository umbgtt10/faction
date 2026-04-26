// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

use crate::cluster_readiness_output::ClusterReadinessOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputBatch {
    outputs: Vec<ClusterReadinessOutput>,
}

impl OutputBatch {
    #[must_use]
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
        }
    }

    #[must_use]
    pub fn from(outputs: Vec<ClusterReadinessOutput>) -> Self {
        Self { outputs }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.outputs.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<ClusterReadinessOutput> {
        self.outputs.get(index).copied()
    }

    #[must_use]
    pub fn outputs(&self) -> &[ClusterReadinessOutput] {
        &self.outputs
    }
}

impl Default for OutputBatch {
    fn default() -> Self {
        Self::new()
    }
}
