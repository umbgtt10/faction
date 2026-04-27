// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_machine_observer::NoOpMachineObserver;
use faction::quorum_policy::QuorumPolicy;

use faction::machine::Machine;
use faction::machine_config::MachineConfig;
use faction::machine_input::MachineInput;
use faction::machine_output::MachineOutput;
use faction::machine_snapshot::MachineSnapshot;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::state_snapshot::StateSnapshot;
use faction::states::initial::Initial;

fn test_machine() -> Machine {
    Machine::new(
        MachineConfig::new(
            0,
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(4),
            FreshnessPolicy::new(2),
        ),
        Box::new(NoOpMachineObserver),
    )
}

#[test]
fn deal_accepts_participation_observed() {
    let mut machine = test_machine();
    let outputs = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let snap = machine.snapshot();
    assert_eq!(
        outputs,
        vec![MachineOutput::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.phase1_confirmed_count(), 1);
}

#[test]
fn deal_accepts_ready_observed() {
    let mut machine = test_machine();
    let outputs = machine.apply(MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let snap = machine.snapshot();
    assert_eq!(outputs, vec![MachineOutput::ReadyAccepted { peer_id: 1 }]);
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.phase2_confirmed_count(), 1);
}

#[test]
fn deal_rejects_local_participation_completed() {
    let mut machine = test_machine();
    let outputs = machine.apply(MachineInput::LocalParticipationCompleted);
    let snap = machine.snapshot();
    assert!(outputs.is_empty());
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.phase2_confirmed_count(), 0);
}

#[test]
fn deal_rejects_deadline_expired() {
    let mut machine = test_machine();
    let outputs = machine.apply(MachineInput::DeadlineExpired);
    let snap = machine.snapshot();
    assert!(outputs.is_empty());
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.exit_mode(), None);
}

#[test]
fn stays_in_initial_after_rejected_input() {
    let mut machine = test_machine();
    let first = machine.apply(MachineInput::DeadlineExpired);
    let second = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let snap = machine.snapshot();
    assert!(first.is_empty());
    assert_eq!(
        second,
        vec![MachineOutput::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snap.phase1_confirmed_count(), 1);
}

#[test]
fn multiple_rejected_inputs_keep_initial_unchanged() {
    let mut machine = test_machine();
    let r1 = machine.apply(MachineInput::LocalParticipationCompleted);
    let r2 = machine.apply(MachineInput::DeadlineExpired);
    let r3 = machine.apply(MachineInput::LocalParticipationCompleted);
    let snap = machine.snapshot();
    assert!(r1.is_empty());
    assert!(r2.is_empty());
    assert!(r3.is_empty());
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.phase1_confirmed_count(), 0);
    assert_eq!(snap.phase2_confirmed_count(), 0);
    assert_eq!(snap.exit_mode(), None);
    assert!(!snap.readiness_exited());
    assert!(!snap.local_participation_complete());
}

#[test]
fn punch_participation_non_member_from_initial() {
    let mut machine = test_machine();
    let outputs = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    });
    let snap = machine.snapshot();
    assert_eq!(
        outputs,
        vec![MachineOutput::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(snap.phase1_confirmed_count(), 0);
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
}

#[test]
fn punch_participation_delayed_from_initial() {
    let mut machine = test_machine();
    let outputs = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 8,
        current_marker: 10,
    });
    let snap = machine.snapshot();
    assert_eq!(
        outputs,
        vec![MachineOutput::DelayedParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snap.phase1_confirmed_count(), 1);
}

#[test]
fn punch_ready_non_member_from_initial() {
    let mut machine = test_machine();
    let outputs = machine.apply(MachineInput::ReadyObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    });
    let snap = machine.snapshot();
    assert_eq!(
        outputs,
        vec![MachineOutput::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(snap.phase2_confirmed_count(), 0);
}

#[test]
fn vibe_check_returns_phase1_active_with_zeros() {
    let machine = test_machine();
    let snap = machine.snapshot();
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.exit_mode(), None);
    assert!(!snap.local_participation_complete());
    assert!(!snap.readiness_exited());
    assert_eq!(snap.phase1_confirmed_count(), 0);
    assert_eq!(snap.phase2_confirmed_count(), 0);
    assert_eq!(snap.quorum_threshold(), 4);
}

#[test]
fn initial_state_snapshot_inherits_correctly() {
    let prev = MachineSnapshot::new(
        ReadinessLifecycleState::Phase2Active,
        Some(ReadinessExitMode::Deadline),
        true,
        true,
        99,
        99,
        4,
    );
    let result = Initial.state_snapshot(&prev);
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(result.phase1_confirmed_count(), 0);
    assert_eq!(result.phase2_confirmed_count(), 0);
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(result.local_participation_complete());
    assert!(result.readiness_exited());
}
