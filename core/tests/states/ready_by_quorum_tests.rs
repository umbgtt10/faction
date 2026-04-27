// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::freshness_policy::FreshnessPolicy;
use faction::machine::Machine;
use faction::machine_config::MachineConfig;
use faction::machine_input::MachineInput;
use faction::no_op_machine_observer::NoOpMachineObserver;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;

fn reach_ready_by_quorum() -> Machine {
    let mut machine = Machine::new(
        MachineConfig::new(
            0,
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(4),
            FreshnessPolicy::new(2),
        ),
        Box::new(NoOpMachineObserver),
    );
    let _ = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = machine.apply(MachineInput::LocalParticipationCompleted);
    let _ = machine.apply(MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = machine.apply(MachineInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = machine.apply(MachineInput::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    });
    machine
}

#[test]
fn deal_rejects_participation_observed() {
    // Arrange
    let machine = reach_ready_by_quorum();
    let snapshot = machine.snapshot();

    // Assert
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot.readiness_exited());
}

#[test]
fn all_inputs_leave_state_unchanged() {
    // Arrange
    let mut machine = reach_ready_by_quorum();
    let snapshot_before = machine.snapshot();

    // Act
    let r1 = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 0,
        freshness: 10,
        current_marker: 10,
    });
    let r2 = machine.apply(MachineInput::ReadyObserved {
        peer_id: 4,
        freshness: 10,
        current_marker: 10,
    });
    let r3 = machine.apply(MachineInput::LocalParticipationCompleted);
    let r4 = machine.apply(MachineInput::DeadlineExpired);
    let snapshot_after = machine.snapshot();

    // Assert
    assert!(r1.is_empty());
    assert!(r2.is_empty());
    assert!(r3.is_empty());
    assert!(r4.is_empty());
    assert_eq!(snapshot_before, snapshot_after);
}

#[test]
fn vibe_check_returns_correct_snapshot() {
    // Arrange & Act
    let machine = reach_ready_by_quorum();
    let snapshot = machine.snapshot();

    // Assert
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot.local_participation_complete());
    assert!(snapshot.readiness_exited());
    assert_eq!(snapshot.phase1_confirmed_count(), 1);
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
    assert_eq!(snapshot.quorum_threshold(), 4);
}
