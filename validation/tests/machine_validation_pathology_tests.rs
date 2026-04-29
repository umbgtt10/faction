// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::outcome::Outcome;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction_validation::machine_scenario_harness::MachineScenarioHarness;

#[test]
fn stale_signals_do_not_perturb_active_multi_node_state() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 10);

    // Act
    let outputs = harness.apply_ready(0, 2, 7);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(outputs, vec![Outcome::StaleReadyIgnored { peer_id: 2 }]);
    assert_eq!(snapshot.phase2_confirmed_count(), 2);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn duplicate_signals_across_nodes_remain_idempotent() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 10);

    // Act
    let outputs = harness.apply_ready(0, 1, 10);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(outputs, vec![Outcome::DuplicateReadyIgnored { peer_id: 1 }]);
    assert_eq!(snapshot.phase2_confirmed_count(), 2);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn mixed_delayed_stale_and_duplicate_sequence_preserves_correct_state() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 10);

    // Act
    let delayed_outputs = harness.apply_ready(0, 2, 8);
    let stale_outputs = harness.apply_ready(0, 3, 7);
    let duplicate_outputs = harness.apply_ready(0, 1, 10);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(
        delayed_outputs,
        vec![Outcome::DelayedReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(
        stale_outputs,
        vec![Outcome::StaleReadyIgnored { peer_id: 3 }]
    );
    assert_eq!(
        duplicate_outputs,
        vec![Outcome::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 3);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn observability_trace_captures_accept_ignore_delay_and_exit_decisions() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);

    // Act
    let step1 = harness.apply_ready(0, 1, 10);
    let step2 = harness.apply_ready(0, 2, 8);
    let step3 = harness.apply_ready(0, 3, 7);
    let step4 = harness.apply_ready(0, 1, 10);
    let step5 = harness.apply_ready(0, 3, 10);

    // Assert
    assert_eq!(step1, vec![Outcome::ReadyAccepted { peer_id: 1 }]);
    assert_eq!(step2, vec![Outcome::DelayedReadyAccepted { peer_id: 2 }]);
    assert_eq!(step3, vec![Outcome::StaleReadyIgnored { peer_id: 3 }]);
    assert_eq!(step4, vec![Outcome::DuplicateReadyIgnored { peer_id: 1 }]);
    assert_eq!(
        step5,
        vec![
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::ReadyQuorumReached,
            Outcome::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    let snapshot = harness.snapshot(0);
    assert!(snapshot.readiness_exited());
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
}

#[test]
fn non_member_signal_does_not_perturb_multi_node_state() {
    // Arrange
    let mut harness = MachineScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 10);

    // Act
    let outputs = harness.apply_ready(0, 99, 10);
    let snapshot = harness.snapshot(0);

    // Assert
    assert_eq!(outputs, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(snapshot.phase2_confirmed_count(), 2);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}
