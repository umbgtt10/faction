// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::cluster_readiness::ClusterReadiness;
use faction::cluster_readiness_config::ClusterReadinessConfig;
use faction::cluster_readiness_input::ClusterReadinessInput;
use faction::cluster_readiness_output::ClusterReadinessOutput;
use faction::cluster_readiness_snapshot::ClusterReadinessSnapshot;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_cluster_readiness_observer::NoOpClusterReadinessObserver;
use faction::quorum_policy::QuorumPolicy;
use faction::PeerId;

pub struct ClusterReadinessScenarioHarness {
    coordinators: Vec<ClusterReadiness>,
    current_marker: u64,
}

impl ClusterReadinessScenarioHarness {
    pub fn new(peer_set: Vec<PeerId>, quorum_threshold: usize, max_delay: u64) -> Self {
        let mut coordinators = Vec::new();

        for peer_id in peer_set.iter().copied() {
            coordinators.push(ClusterReadiness::new(
                ClusterReadinessConfig::new(
                    peer_id,
                    peer_set.clone(),
                    QuorumPolicy::new(quorum_threshold),
                    FreshnessPolicy::new(max_delay),
                ),
                Box::new(NoOpClusterReadinessObserver),
            ));
        }

        Self {
            coordinators,
            current_marker: 0,
        }
    }

    pub fn current_marker(&self) -> u64 {
        self.current_marker
    }

    pub fn advance_to(&mut self, marker: u64) {
        self.current_marker = marker;
    }

    pub fn advance_by(&mut self, delta: u64) {
        self.current_marker += delta;
    }

    pub fn coordinator_count(&self) -> usize {
        self.coordinators.len()
    }

    pub fn snapshot(&self, coordinator_index: usize) -> ClusterReadinessSnapshot {
        self.coordinators[coordinator_index].snapshot()
    }

    pub fn apply_participation(
        &mut self,
        coordinator_index: usize,
        peer_id: PeerId,
        freshness: u64,
    ) -> Vec<ClusterReadinessOutput> {
        let batch = self.coordinators[coordinator_index].apply(
            ClusterReadinessInput::ParticipationObserved {
                peer_id,
                freshness,
                current_marker: self.current_marker,
            },
        );

        batch.outputs().to_vec()
    }

    pub fn apply_ready(
        &mut self,
        coordinator_index: usize,
        peer_id: PeerId,
        freshness: u64,
    ) -> Vec<ClusterReadinessOutput> {
        let batch =
            self.coordinators[coordinator_index].apply(ClusterReadinessInput::ReadyObserved {
                peer_id,
                freshness,
                current_marker: self.current_marker,
            });

        batch.outputs().to_vec()
    }

    pub fn complete_local_participation(
        &mut self,
        coordinator_index: usize,
    ) -> Vec<ClusterReadinessOutput> {
        let batch = self.coordinators[coordinator_index]
            .apply(ClusterReadinessInput::LocalParticipationCompleted);

        batch.outputs().to_vec()
    }

    pub fn expire_deadline(&mut self, coordinator_index: usize) -> Vec<ClusterReadinessOutput> {
        let batch =
            self.coordinators[coordinator_index].apply(ClusterReadinessInput::DeadlineExpired);

        batch.outputs().to_vec()
    }
}
