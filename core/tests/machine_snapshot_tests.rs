// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use faction::machine_snapshot::MachineSnapshot;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;

const BASE: MachineSnapshot = MachineSnapshot::new(
    ReadinessLifecycleState::Phase2Active,
    Some(ReadinessExitMode::Deadline),
    true,
    true,
    5,
    7,
    3,
);

#[test]
fn with_lifecycle_state_updates_only_lifecycle_state() {
    let result = BASE.with_lifecycle_state(ReadinessLifecycleState::Phase1Active);
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(result.local_participation_complete());
    assert!(result.readiness_exited());
    assert_eq!(result.phase1_confirmed_count(), 5);
    assert_eq!(result.phase2_confirmed_count(), 7);
    assert_eq!(result.quorum_threshold(), 3);
}

#[test]
fn with_exit_mode_updates_only_exit_mode() {
    let result = BASE.with_exit_mode(Some(ReadinessExitMode::Quorum));
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(result.local_participation_complete());
    assert!(result.readiness_exited());
    assert_eq!(result.phase1_confirmed_count(), 5);
    assert_eq!(result.phase2_confirmed_count(), 7);
    assert_eq!(result.quorum_threshold(), 3);
}

#[test]
fn with_local_participation_complete_updates_only_local_participation() {
    let result = BASE.with_local_participation_complete(false);
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(!result.local_participation_complete());
    assert!(result.readiness_exited());
    assert_eq!(result.phase1_confirmed_count(), 5);
    assert_eq!(result.phase2_confirmed_count(), 7);
    assert_eq!(result.quorum_threshold(), 3);
}

#[test]
fn with_readiness_exited_updates_only_readiness_exited() {
    let result = BASE.with_readiness_exited(false);
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(result.local_participation_complete());
    assert!(!result.readiness_exited());
    assert_eq!(result.phase1_confirmed_count(), 5);
    assert_eq!(result.phase2_confirmed_count(), 7);
    assert_eq!(result.quorum_threshold(), 3);
}

#[test]
fn with_phase1_count_updates_only_phase1_count() {
    let result = BASE.with_phase1_count(42);
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(result.local_participation_complete());
    assert!(result.readiness_exited());
    assert_eq!(result.phase1_confirmed_count(), 42);
    assert_eq!(result.phase2_confirmed_count(), 7);
    assert_eq!(result.quorum_threshold(), 3);
}

#[test]
fn with_phase2_count_updates_only_phase2_count() {
    let result = BASE.with_phase2_count(99);
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(result.local_participation_complete());
    assert!(result.readiness_exited());
    assert_eq!(result.phase1_confirmed_count(), 5);
    assert_eq!(result.phase2_confirmed_count(), 99);
    assert_eq!(result.quorum_threshold(), 3);
}
