// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::apply_status::ApplyStatus;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::snapshot::Snapshot;
use faction::state_snapshot::StateSnapshot;
use faction::states::ready_by_deadline::ReadyByDeadline;

fn make_faction() -> Faction {
    Faction::new(
        Config::new(
            0,
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(4),
            FreshnessPolicy::new(2),
        ),
        Box::new(NoOpObserver),
    )
}

fn reach_deadline_from_phase1() -> Faction {
    let mut f = make_faction();
    let _ = f.apply(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = f.apply(Command::DeadlineExpired);
    f
}

fn reach_deadline_from_phase2() -> Faction {
    let mut f = make_faction();
    let _ = f.apply(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = f.apply(Command::LocalParticipationCompleted);
    let _ = f.apply(Command::DeadlineExpired);
    f
}

#[test]
fn deal_rejects_participation_observed() {
    // Arrange
    let mut f = reach_deadline_from_phase1();
    let snap_before = f.snapshot();

    // Act & Assert
    match f.apply(Command::ParticipationObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    }) {
        ApplyStatus::Rejected { .. } => {}
        ApplyStatus::Accepted { .. } => panic!("expected rejected"),
        ApplyStatus::Snapshot { .. } => unreachable!(),
    };
    assert_eq!(f.snapshot(), snap_before);
}

#[test]
fn deal_rejects_ready_observed() {
    // Arrange
    let mut f = reach_deadline_from_phase1();
    let snap_before = f.snapshot();

    // Act & Assert
    match f.apply(Command::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    }) {
        ApplyStatus::Rejected { .. } => {}
        ApplyStatus::Accepted { .. } => panic!("expected rejected"),
        ApplyStatus::Snapshot { .. } => unreachable!(),
    };
    assert_eq!(f.snapshot(), snap_before);
}

#[test]
fn deal_rejects_local_participation_completed() {
    // Arrange
    let mut f = reach_deadline_from_phase1();
    let snap_before = f.snapshot();

    // Act & Assert
    match f.apply(Command::LocalParticipationCompleted) {
        ApplyStatus::Rejected { .. } => {}
        ApplyStatus::Accepted { .. } => panic!("expected rejected"),
        ApplyStatus::Snapshot { .. } => unreachable!(),
    };
    assert_eq!(f.snapshot(), snap_before);
}

#[test]
fn deal_rejects_deadline_expired() {
    // Arrange
    let mut f = reach_deadline_from_phase1();
    let snap_before = f.snapshot();

    // Act & Assert
    match f.apply(Command::DeadlineExpired) {
        ApplyStatus::Rejected { .. } => {}
        ApplyStatus::Accepted { .. } => panic!("expected rejected"),
        ApplyStatus::Snapshot { .. } => unreachable!(),
    };
    assert_eq!(f.snapshot(), snap_before);
}

#[test]
fn vibe_check_after_deadline_from_phase1() {
    // Arrange & Act
    let f = reach_deadline_from_phase1();
    let s = f.snapshot();

    // Assert
    assert_eq!(
        s.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert_eq!(s.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(s.readiness_exited());
    assert!(!s.local_participation_complete());
    assert_eq!(s.phase1_confirmed_count(), 1);
    assert_eq!(s.phase2_confirmed_count(), 0);
}

#[test]
fn vibe_check_after_deadline_from_phase2() {
    // Arrange & Act
    let f = reach_deadline_from_phase2();
    let s = f.snapshot();

    // Assert
    assert_eq!(
        s.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert_eq!(s.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(s.readiness_exited());
    assert!(s.local_participation_complete());
    assert_eq!(s.phase1_confirmed_count(), 1);
    assert_eq!(s.phase2_confirmed_count(), 1);
}

#[test]
fn post_deadline_inputs_leave_state_unchanged() {
    // Arrange
    let mut f = reach_deadline_from_phase1();
    let snapshot_before = f.snapshot();

    // Act
    let _ = f.apply(Command::ParticipationObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = f.apply(Command::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = f.apply(Command::LocalParticipationCompleted);
    let _ = f.apply(Command::DeadlineExpired);

    // Assert
    assert_eq!(f.snapshot(), snapshot_before);
}

#[test]
fn ready_by_deadline_state_snapshot_inherits_local_completion_from_phase1() {
    // Arrange
    let rbd = ReadyByDeadline {
        phase1_count: 3,
        phase2_count: 1,
    };
    let prev = Snapshot::new(
        ReadinessLifecycleState::Phase1Active,
        Some(ReadinessExitMode::Deadline),
        false,
        false,
        99,
        99,
        4,
    );

    // Act
    let result = rbd.state_snapshot(&prev);

    // Assert
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(result.readiness_exited());
    assert!(!result.local_participation_complete());
    assert_eq!(result.phase1_confirmed_count(), 3);
    assert_eq!(result.phase2_confirmed_count(), 1);
    assert_eq!(result.quorum_threshold(), 4);
}

#[test]
fn ready_by_deadline_state_snapshot_inherits_local_completion_from_phase2() {
    // Arrange
    let rbd = ReadyByDeadline {
        phase1_count: 2,
        phase2_count: 4,
    };
    let prev = Snapshot::new(
        ReadinessLifecycleState::Phase2Active,
        Some(ReadinessExitMode::Deadline),
        true,
        false,
        99,
        99,
        4,
    );

    // Act
    let result = rbd.state_snapshot(&prev);

    // Assert
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(result.readiness_exited());
    assert!(result.local_participation_complete());
    assert_eq!(result.phase1_confirmed_count(), 2);
    assert_eq!(result.phase2_confirmed_count(), 4);
    assert_eq!(result.quorum_threshold(), 4);
}
