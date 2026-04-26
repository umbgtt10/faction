// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::vibe_output::VibeOutput;
use faction_validation::vibe_scenario_harness::VibeScenarioHarness;

#[test]
fn complete_local_participation_updates_snapshot_and_outputs() {
    // Arrange
    let mut harness = VibeScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);

    // Act
    let outputs = harness.complete_local_participation(0);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(
        outputs,
        vec![
            VibeOutput::LocalParticipationCompleted,
            VibeOutput::BroadcastLocalReady,
        ]
    );
    assert!(snapshot.local_participation_complete());
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 1);
    assert!(!snapshot.readiness_exited());
}

#[test]
fn apply_participation_accepts_timely_member_observation() {
    // Arrange
    let mut harness = VibeScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);

    // Act
    let outputs = harness.apply_participation(0, 1, 10);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(
        outputs,
        vec![VibeOutput::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 1);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn apply_participation_accepts_delayed_member_observation_within_margin() {
    // Arrange
    let mut harness = VibeScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);

    // Act
    let outputs = harness.apply_participation(0, 1, 8);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(
        outputs,
        vec![VibeOutput::DelayedParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 1);
    assert!(!snapshot.readiness_exited());
}

#[test]
fn apply_participation_rejects_stale_member_observation() {
    // Arrange
    let mut harness = VibeScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);

    // Act
    let outputs = harness.apply_participation(0, 1, 7);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(
        outputs,
        vec![VibeOutput::StaleParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 0);
    assert_eq!(snapshot.phase2_confirmed_count(), 0);
    assert!(!snapshot.readiness_exited());
}
