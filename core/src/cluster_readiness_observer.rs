// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::cluster_readiness_input::ClusterReadinessInput;
use crate::cluster_readiness_transition::ClusterReadinessTransition;

pub trait ClusterReadinessObserver {
    fn observe(&mut self, input: ClusterReadinessInput, transition: ClusterReadinessTransition);
}
