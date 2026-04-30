// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use faction::cluster_view::ClusterView;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;

const BASE: ClusterView = ClusterView::new(ReadinessLifecycleState::Bootstrapped, true, 5, 7, 3);

#[test]
fn with_lifecycle_state_updates_only_lifecycle_state() {
    // Arrange & Act
    let result = BASE.with_lifecycle_state(ReadinessLifecycleState::Phase1Active);

    // Assert
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(result.exit_mode(), None);
    assert!(result.local_participation_complete());
    assert!(!result.readiness_exited());
    assert_eq!(result.phase1_confirmed_count(), 5);
    assert_eq!(result.phase2_confirmed_count(), 7);
    assert_eq!(result.quorum_threshold(), 3);
}

#[test]
fn with_phase2_count_updates_only_phase2_count() {
    // Arrange & Act
    let result = BASE.with_phase2_count(99);

    // Assert
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Bootstrapped
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Bootstrapped));
    assert!(result.local_participation_complete());
    assert!(result.readiness_exited());
    assert_eq!(result.phase1_confirmed_count(), 5);
    assert_eq!(result.phase2_confirmed_count(), 99);
    assert_eq!(result.quorum_threshold(), 3);
}
