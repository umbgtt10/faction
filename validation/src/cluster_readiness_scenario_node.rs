// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::cluster_readiness::ClusterReadiness;
use faction::cluster_readiness_config::ClusterReadinessConfig;
use faction::cluster_readiness_observer::ClusterReadinessObserver;
use faction::cluster_readiness_output::ClusterReadinessOutput;
use faction::cluster_readiness_snapshot::ClusterReadinessSnapshot;
use faction::no_op_cluster_readiness_observer::NoOpClusterReadinessObserver;
use faction::PeerId;

pub struct ClusterReadinessScenarioNode {
    peer_id: PeerId,
    readiness: ClusterReadiness,
}

impl ClusterReadinessScenarioNode {
    #[must_use]
    pub fn new(peer_id: PeerId, config: ClusterReadinessConfig) -> Self {
        Self {
            peer_id,
            readiness: ClusterReadiness::new(config, Box::new(NoOpClusterReadinessObserver)),
        }
    }

    #[must_use]
    pub fn new_with_observer(
        peer_id: PeerId,
        config: ClusterReadinessConfig,
        observer: Box<dyn ClusterReadinessObserver>,
    ) -> Self {
        Self {
            peer_id,
            readiness: ClusterReadiness::new(config, observer),
        }
    }

    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    #[must_use]
    pub fn config(&self) -> &ClusterReadinessConfig {
        self.readiness.config()
    }

    #[must_use]
    pub fn snapshot(&self) -> ClusterReadinessSnapshot {
        self.readiness.snapshot()
    }

    #[must_use]
    pub fn apply(
        self,
        input: faction::cluster_readiness_input::ClusterReadinessInput,
    ) -> (Self, Vec<ClusterReadinessOutput>) {
        let mut readiness = self.readiness;
        let outputs = readiness.apply(input);

        (
            Self {
                peer_id: self.peer_id,
                readiness,
            },
            outputs,
        )
    }
}
