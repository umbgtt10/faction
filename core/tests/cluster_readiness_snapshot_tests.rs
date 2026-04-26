// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::cluster_readiness::ClusterReadiness;
use faction::cluster_readiness_config::ClusterReadinessConfig;
use faction::freshness_policy::FreshnessPolicy;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;

fn test_config() -> ClusterReadinessConfig {
    ClusterReadinessConfig::new(
        0,
        vec![0, 1, 2, 3, 4],
        QuorumPolicy::new(4),
        FreshnessPolicy::new(2),
    )
}

#[test]
fn snapshot_reflects_initial_state() {
    // Arrange & Act
    let coordinator = ClusterReadiness::new(
        test_config(),
        alloc::boxed::Box::new(
            faction::no_op_cluster_readiness_observer::NoOpClusterReadinessObserver,
        ),
    );
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snapshot.exit_mode(), None);
    assert!(!snapshot.local_participation_complete());
    assert!(!snapshot.readiness_exited());
    assert_eq!(snapshot.phase1_confirmed_count(), 0);
    assert_eq!(snapshot.phase2_confirmed_count(), 0);
    assert_eq!(snapshot.quorum_threshold(), 4);
}
