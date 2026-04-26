// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::cluster_readiness_input::ClusterReadinessInput;
use crate::cluster_readiness_observer::ClusterReadinessObserver;
use crate::cluster_readiness_transition::ClusterReadinessTransition;

pub struct NoOpClusterReadinessObserver;

impl ClusterReadinessObserver for NoOpClusterReadinessObserver {
    fn observe(&mut self, _input: ClusterReadinessInput, _transition: ClusterReadinessTransition) {}
}
