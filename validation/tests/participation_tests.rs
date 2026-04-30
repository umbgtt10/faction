// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;
use faction::outcome::Outcome;
use faction::node_state::NodeState;
use faction_validation::scenario_harness::ScenarioHarness;

#[test]
fn complete_local_participation_updates_snapshot_and_outputs() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);

    // Act
    let outputs = harness.complete_local_participation(0);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(
        outputs,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
    assert!(cluster_view.local_participation_complete());
    assert_eq!(
        cluster_view.node_state(),
        NodeState::Phase2Active
    );
    assert_eq!(cluster_view.phase2_confirmed_count(), 1);
    assert!(!cluster_view.readiness_exited());
}

#[test]
fn apply_participation_accepts_timely_member_observation() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);

    // Act
    let outputs = harness.apply_participation(0, 1, 10);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(outputs, vec![Outcome::ParticipationAccepted { peer_id: 1 }]);
    assert_eq!(cluster_view.phase1_confirmed_count(), 1);
    assert_eq!(
        cluster_view.node_state(),
        NodeState::Phase1Active
    );
    assert!(!cluster_view.readiness_exited());
}

#[test]
fn apply_participation_accepts_delayed_member_observation_within_margin() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);

    // Act
    let outputs = harness.apply_participation(0, 1, 8);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(
        outputs,
        vec![Outcome::DelayedParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(cluster_view.phase1_confirmed_count(), 1);
    assert!(!cluster_view.readiness_exited());
}

#[test]
fn apply_participation_rejects_stale_member_observation() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);

    // Act
    let outputs = harness.apply_participation(0, 1, 7);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(
        outputs,
        vec![Outcome::StaleParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(cluster_view.phase1_confirmed_count(), 0);
    assert_eq!(cluster_view.phase2_confirmed_count(), 0);
    assert!(!cluster_view.readiness_exited());
}
