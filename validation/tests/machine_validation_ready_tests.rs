// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::machine_output::MachineOutput;
use faction_validation::machine_scenario_harness::MachineScenarioHarness;

#[test]
fn apply_ready_accepts_timely_member_observation_after_local_completion() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);

    // Act
    let outputs = harness.apply_ready(0, 1, 10);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(outputs, vec![MachineOutput::ReadyAccepted { peer_id: 1 }]);
    assert_eq!(snapshot.phase2_confirmed_count(), 2);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn apply_ready_accepts_delayed_member_observation_within_margin() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);

    // Act
    let outputs = harness.apply_ready(0, 1, 8);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(
        outputs,
        vec![MachineOutput::DelayedReadyAccepted { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 2);
    assert!(!snapshot.readiness_exited());
}

#[test]
fn apply_ready_rejects_stale_member_observation() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);

    // Act
    let outputs = harness.apply_ready(0, 1, 7);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(outputs, vec![MachineOutput::StaleReadyIgnored { peer_id: 1 }]);
    assert_eq!(snapshot.phase2_confirmed_count(), 1);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn apply_ready_reaches_quorum_exit_in_asymmetric_startup_sequence() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
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
            MachineOutput::ReadyAccepted { peer_id: 3 },
            MachineOutput::ReadyQuorumReached,
            MachineOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert!(snapshot.readiness_exited());
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
}

#[test]
fn delayed_signals_within_margin_still_allow_quorum_exit() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 8);
    let _ = harness.apply_ready(0, 2, 9);

    // Act
    let outputs = harness.apply_ready(0, 3, 8);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(
        outputs,
        vec![
            MachineOutput::DelayedReadyAccepted { peer_id: 3 },
            MachineOutput::ReadyQuorumReached,
            MachineOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
    assert!(snapshot.readiness_exited());
}

#[test]
fn post_exit_ready_is_ignored() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 10);
    let _ = harness.apply_ready(0, 2, 10);
    let _ = harness.apply_ready(0, 3, 10);

    // Act
    let outputs = harness.apply_ready(0, 4, 10);
    let snapshot = harness.snapshot(0);

    // Assert
    assert!(outputs.is_empty());
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
    assert!(snapshot.readiness_exited());
}

#[test]
fn five_node_asymmetric_startup_reaches_quorum_exit() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(5);
    let _ = harness.complete_local_participation(2);
    let _ = harness.complete_local_participation(3);
    harness.advance_to(8);
    let _ = harness.complete_local_participation(1);
    let _ = harness.complete_local_participation(4);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 10);
    let _ = harness.apply_ready(0, 2, 10);
    let _ = harness.apply_ready(1, 0, 10);
    let _ = harness.apply_ready(1, 2, 10);

    // Act
    let outputs_0 = harness.apply_ready(0, 3, 10);
    let outputs_1 = harness.apply_ready(1, 3, 10);

    // Assert
    assert_eq!(
        outputs_0,
        vec![
            MachineOutput::ReadyAccepted { peer_id: 3 },
            MachineOutput::ReadyQuorumReached,
            MachineOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(
        outputs_1,
        vec![
            MachineOutput::ReadyAccepted { peer_id: 3 },
            MachineOutput::ReadyQuorumReached,
            MachineOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    let snapshot_0 = harness.snapshot(0);
    let snapshot_1 = harness.snapshot(1);
    assert_eq!(snapshot_0.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert_eq!(snapshot_1.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot_0.readiness_exited());
    assert!(snapshot_1.readiness_exited());
    assert_eq!(
        snapshot_0.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert_eq!(
        snapshot_1.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
}

#[test]
fn early_ready_signals_accumulate_before_local_participation_completion() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let outputs_peer_1 = harness.apply_ready(0, 1, 10);
    let outputs_peer_2 = harness.apply_ready(0, 2, 10);
    let outputs_peer_3 = harness.apply_ready(0, 3, 10);
    let intermediate_snapshot = harness.snapshot(0);

    // Act
    let outputs = harness.complete_local_participation(0);

    // Assert
    assert_eq!(
        outputs_peer_1,
        vec![MachineOutput::ReadyAccepted { peer_id: 1 }]
    );
    assert_eq!(
        outputs_peer_2,
        vec![MachineOutput::ReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(
        outputs_peer_3,
        vec![MachineOutput::ReadyAccepted { peer_id: 3 }]
    );
    assert_eq!(intermediate_snapshot.phase2_confirmed_count(), 3);
    assert_eq!(
        intermediate_snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert!(!intermediate_snapshot.local_participation_complete());
    assert!(!intermediate_snapshot.readiness_exited());
    assert_eq!(
        outputs,
        vec![
            MachineOutput::LocalParticipationCompleted,
            MachineOutput::BroadcastLocalReady,
            MachineOutput::ReadyQuorumReached,
            MachineOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    let snapshot = harness.snapshot(0);
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot.readiness_exited());
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
}
