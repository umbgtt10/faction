// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::outcome::Outcome;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::snapshot::Snapshot;
use faction::transition::Transition;

fn snapshot(phase1: usize, phase2: usize) -> Snapshot {
    Snapshot::new(
        ReadinessLifecycleState::Phase1Active,
        None,
        false,
        false,
        phase1,
        phase2,
        4,
    )
}

fn snapshot_exited() -> Snapshot {
    Snapshot::new(
        ReadinessLifecycleState::Bootstrapped,
        Some(ReadinessExitMode::Quorum),
        true,
        true,
        3,
        5,
        4,
    )
}

#[test]
fn new_stores_previous_state() {
    // Arrange
    let prev = snapshot(1, 0);
    let next = snapshot(1, 1);
    let outputs = vec![Outcome::ParticipationAccepted { peer_id: 1 }];

    // Act
    let transition = Transition::new(prev, outputs, next);

    // Assert
    assert_eq!(transition.previous_state(), snapshot(1, 0));
}

#[test]
fn new_stores_new_state() {
    // Arrange
    let prev = snapshot(1, 0);
    let next = snapshot(1, 1);
    let outputs = vec![Outcome::ParticipationAccepted { peer_id: 1 }];

    // Act
    let transition = Transition::new(prev, outputs, next);

    // Assert
    assert_eq!(transition.new_state(), snapshot(1, 1));
}

#[test]
fn new_stores_outputs() {
    // Arrange
    let prev = snapshot(0, 0);
    let next = snapshot(1, 0);
    let outputs = vec![
        Outcome::LocalParticipationCompleted,
        Outcome::BroadcastLocalReady,
    ];

    // Act
    let transition = Transition::new(prev, outputs.clone(), next);

    // Assert
    assert_eq!(transition.outputs(), &outputs);
}

#[test]
fn new_handles_empty_outputs() {
    // Arrange
    let prev = snapshot(0, 0);
    let next = snapshot(0, 1);
    let outputs = vec![];

    // Act
    let transition = Transition::new(prev, outputs.clone(), next);

    // Assert
    assert!(transition.outputs().is_empty());
}

#[test]
fn previous_state_preserves_full_snapshot() {
    // Arrange
    let prev = snapshot_exited();
    let next = snapshot(0, 0);
    let outputs = vec![];

    // Act
    let transition = Transition::new(prev, outputs, next);

    // Assert
    let result = transition.previous_state();
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Bootstrapped
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(result.local_participation_complete());
    assert!(result.readiness_exited());
    assert_eq!(result.phase1_confirmed_count(), 3);
    assert_eq!(result.phase2_confirmed_count(), 5);
}

#[test]
fn new_state_preserves_full_snapshot() {
    // Arrange
    let prev = snapshot(0, 0);
    let next = snapshot_exited();

    // Act
    let transition = Transition::new(prev, vec![], next);

    // Assert
    let result = transition.new_state();
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Bootstrapped
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(result.local_participation_complete());
    assert!(result.readiness_exited());
    assert_eq!(result.phase1_confirmed_count(), 3);
    assert_eq!(result.phase2_confirmed_count(), 5);
}

#[test]
fn outputs_are_immutable() {
    // Arrange
    let prev = snapshot(0, 0);
    let next = snapshot(1, 0);
    let outputs = vec![Outcome::ReadyAccepted { peer_id: 1 }];

    // Act
    let transition = Transition::new(prev, outputs, next);

    // Assert
    assert_eq!(transition.outputs().len(), 1);
    assert_eq!(
        transition.outputs()[0],
        Outcome::ReadyAccepted { peer_id: 1 }
    );
}

#[test]
fn clone_produces_equal_transition() {
    // Arrange
    let prev = snapshot(1, 0);
    let next = snapshot(1, 2);
    let outputs = vec![
        Outcome::ReadyAccepted { peer_id: 3 },
        Outcome::ReadyQuorumReached,
        Outcome::ReadinessExited {
            mode: ReadinessExitMode::Quorum,
        },
    ];
    let transition = Transition::new(prev, outputs.clone(), next);

    // Act
    let cloned = transition.clone();

    // Assert
    assert_eq!(cloned.previous_state(), transition.previous_state());
    assert_eq!(cloned.new_state(), transition.new_state());
    assert_eq!(cloned.outputs(), transition.outputs());
}

#[test]
fn debug_format_does_not_panic() {
    // Arrange
    let transition = Transition::new(snapshot(0, 0), vec![], snapshot(1, 1));

    // Act & Assert
    let _ = format!("{:?}", transition);
}
