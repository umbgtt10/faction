// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::vibe_output::VibeOutput;
use faction_validation::vibe_scenario_harness::VibeScenarioHarness;

#[test]
fn slow_member_does_not_block_quorum_exit() {
    // Arrange
    let mut harness = VibeScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 10);
    let _ = harness.apply_ready(0, 2, 10);

    // Act
    let outputs = harness.apply_ready(0, 3, 10);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(
        outputs,
        vec![
            VibeOutput::ReadyAccepted { peer_id: 3 },
            VibeOutput::ReadyQuorumReached,
            VibeOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
    assert!(snapshot.readiness_exited());
}

#[test]
fn expire_deadline_exits_by_deadline() {
    // Arrange
    let mut harness = VibeScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    let _ = harness.complete_local_participation(0);

    // Act
    let outputs = harness.expire_deadline(0);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(
        outputs,
        vec![VibeOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline,
        }]
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert!(snapshot.readiness_exited());
}

#[test]
fn post_exit_ready_is_ignored() {
    // Arrange
    let mut harness = VibeScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 10);
    let _ = harness.apply_ready(0, 2, 10);
    let _ = harness.apply_ready(0, 3, 10);

    // Act
    let outputs = harness.apply_ready(0, 4, 10);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(outputs, vec![VibeOutput::StaleReadyIgnored { peer_id: 4 }]);
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
    assert!(snapshot.readiness_exited());
}

#[test]
fn repeated_deadline_expiry_remains_idempotent() {
    // Arrange
    let mut harness = VibeScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    let _ = harness.complete_local_participation(0);
    let _ = harness.expire_deadline(0);

    // Act
    let outputs = harness.expire_deadline(0);
    let snapshot = harness.snapshot(0);

    // Assert
    assert!(outputs.is_empty());
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert!(snapshot.readiness_exited());
}

#[test]
fn deadline_fallback_preserves_progress_when_quorum_never_forms() {
    // Arrange
    let mut harness = VibeScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.complete_local_participation(1);
    let _ = harness.complete_local_participation(2);
    let _ = harness.complete_local_participation(3);
    let _ = harness.complete_local_participation(4);
    let _ = harness.apply_ready(0, 1, 10);
    let _ = harness.apply_ready(0, 2, 10);

    // Act
    let outputs = harness.expire_deadline(0);

    // Assert
    assert_eq!(
        outputs,
        vec![VibeOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline,
        }]
    );
    let snapshot = harness.snapshot(0);
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert!(snapshot.readiness_exited());
    assert_eq!(snapshot.phase2_confirmed_count(), 3);
}

#[test]
fn post_exit_ready_signals_are_harmless_across_multiple_nodes() {
    // Arrange
    let mut harness = VibeScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    for i in 0..5 {
        let _ = harness.complete_local_participation(i);
    }
    let _ = harness.apply_ready(0, 1, 10);
    let _ = harness.apply_ready(0, 2, 10);
    let _ = harness.apply_ready(0, 3, 10);
    let _ = harness.apply_ready(1, 0, 10);
    let _ = harness.apply_ready(1, 2, 10);
    let _ = harness.apply_ready(1, 3, 10);
    let _ = harness.apply_ready(2, 0, 10);
    let _ = harness.apply_ready(2, 1, 10);
    let _ = harness.apply_ready(2, 3, 10);

    // Act
    let outputs_0 = harness.apply_ready(0, 4, 10);
    let outputs_1 = harness.apply_ready(1, 4, 10);
    let outputs_2 = harness.apply_ready(2, 4, 10);

    // Assert
    assert_eq!(
        outputs_0,
        vec![VibeOutput::StaleReadyIgnored { peer_id: 4 }]
    );
    assert_eq!(
        outputs_1,
        vec![VibeOutput::StaleReadyIgnored { peer_id: 4 }]
    );
    assert_eq!(
        outputs_2,
        vec![VibeOutput::StaleReadyIgnored { peer_id: 4 }]
    );
    let snapshot_0 = harness.snapshot(0);
    let snapshot_1 = harness.snapshot(1);
    let snapshot_2 = harness.snapshot(2);
    assert_eq!(snapshot_0.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot_0.readiness_exited());
    assert_eq!(snapshot_1.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot_1.readiness_exited());
    assert_eq!(snapshot_2.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot_2.readiness_exited());
}
