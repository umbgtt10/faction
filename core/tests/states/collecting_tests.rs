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
use faction::outcome::Outcome;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::snapshot::Snapshot;
use faction::state_snapshot::StateSnapshot;
use faction::states::collecting::Collecting;
use faction::states::helpers::confirmed_set::ConfirmedSet;
use faction::Freshness;
use faction::PeerId;

const THRESHOLD: usize = 4;
const MAX_DELAY: Freshness = 2;
const MARKER: Freshness = 10;
const TIMELY: Freshness = 10;
const DELAYED: Freshness = 8;
const STALE: Freshness = 7;

fn machine_in_phase2() -> Faction {
    let mut v = Faction::new(
        Config::new(
            0,
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(THRESHOLD),
            FreshnessPolicy::new(MAX_DELAY),
        ),
        Box::new(NoOpObserver),
    );
    let _ = v.apply(Command::ParticipationObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = v.apply(Command::LocalParticipationCompleted);
    v
}

fn participation(peer_id: PeerId, freshness: Freshness) -> Command {
    Command::ParticipationObserved {
        peer_id,
        freshness,
        current_marker: MARKER,
    }
}

fn ready(peer_id: PeerId, freshness: Freshness) -> Command {
    Command::ReadyObserved {
        peer_id,
        freshness,
        current_marker: MARKER,
    }
}

#[test]
fn deal_accepts_ready_observed() {
    // Arrange & Act
    let mut v = machine_in_phase2();
    let outcomes = match v.apply(ready(1, TIMELY)) {
        ApplyStatus::Accepted { outcomes, .. } => outcomes,
        ApplyStatus::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::ReadyAccepted { peer_id: 1 }]);
}

#[test]
fn deal_accepts_deadline_expired() {
    // Arrange & Act
    let mut v = machine_in_phase2();
    let outcomes = match v.apply(Command::DeadlineExpired) {
        ApplyStatus::Accepted { outcomes, .. } => outcomes,
        ApplyStatus::Rejected { .. } => panic!("expected accepted"),
    };
    let snap = v.snapshot();

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::ReadinessExited {
            mode: ReadinessExitMode::Deadline,
        }]
    );
    assert_eq!(snap.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(snap.readiness_exited());
}

#[test]
fn deal_rejects_participation_observed() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();

    // Act
    assert!(matches!(
        v.apply(participation(2, TIMELY)),
        ApplyStatus::Rejected { .. }
    ));
    assert_eq!(v.snapshot(), snap_before);
}

#[test]
fn deal_rejects_local_participation_completed() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();

    // Act
    assert!(matches!(
        v.apply(Command::LocalParticipationCompleted),
        ApplyStatus::Rejected { .. }
    ));
    assert_eq!(v.snapshot(), snap_before);
}

#[test]
fn participation_non_member_is_noop() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();

    // Act & Assert
    assert!(matches!(
        v.apply(participation(99, TIMELY)),
        ApplyStatus::Rejected { .. }
    ));
    assert_eq!(v.snapshot(), snap_before);
}

#[test]
fn participation_stale_is_noop() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();

    // Act & Assert
    assert!(matches!(
        v.apply(participation(1, STALE)),
        ApplyStatus::Rejected { .. }
    ));
    assert_eq!(v.snapshot(), snap_before);
}

#[test]
fn participation_first_timely_is_noop() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();

    // Act & Assert
    assert!(matches!(
        v.apply(participation(2, TIMELY)),
        ApplyStatus::Rejected { .. }
    ));
    assert_eq!(v.snapshot(), snap_before);
}

#[test]
fn participation_first_delayed_is_noop() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();

    // Act & Assert
    assert!(matches!(
        v.apply(participation(2, DELAYED)),
        ApplyStatus::Rejected { .. }
    ));
    assert_eq!(v.snapshot(), snap_before);
}

#[test]
fn ready_non_member_rejected() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();

    // Act
    let outcomes = match v.apply(ready(99, TIMELY)) {
        ApplyStatus::Accepted { outcomes, .. } => outcomes,
        ApplyStatus::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(v.snapshot(), snap_before);
}

#[test]
fn ready_stale_rejected() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();

    // Act
    let outcomes = match v.apply(ready(1, STALE)) {
        ApplyStatus::Accepted { outcomes, .. } => outcomes,
        ApplyStatus::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::StaleReadyIgnored { peer_id: 1 }]);
    assert_eq!(v.snapshot(), snap_before);
}

#[test]
fn ready_duplicate_rejected() {
    // Arrange
    let mut v = machine_in_phase2();
    let _ = v.apply(ready(1, TIMELY));
    let snap_before = v.snapshot();

    // Act
    let outcomes = match v.apply(ready(1, TIMELY)) {
        ApplyStatus::Accepted { outcomes, .. } => outcomes,
        ApplyStatus::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(v.snapshot(), snap_before);
}

#[test]
fn ready_first_timely_no_quorum() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();
    assert_eq!(snap_before.phase2_confirmed_count(), 1);
    assert_eq!(
        snap_before.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );

    // Act
    let outcomes = match v.apply(ready(1, TIMELY)) {
        ApplyStatus::Accepted { outcomes, .. } => outcomes,
        ApplyStatus::Rejected { .. } => panic!("expected accepted"),
    };
    let snap = v.snapshot();

    // Assert
    assert_eq!(outcomes, vec![Outcome::ReadyAccepted { peer_id: 1 }]);
    assert_eq!(snap.phase2_confirmed_count(), 2);
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snap.readiness_exited());
}

#[test]
fn ready_first_delayed_no_quorum() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();
    assert_eq!(snap_before.phase2_confirmed_count(), 1);

    // Act
    let outcomes = match v.apply(ready(1, DELAYED)) {
        ApplyStatus::Accepted { outcomes, .. } => outcomes,
        ApplyStatus::Rejected { .. } => panic!("expected accepted"),
    };
    let snap = v.snapshot();

    // Assert
    assert_eq!(outcomes, vec![Outcome::DelayedReadyAccepted { peer_id: 1 }]);
    assert_eq!(snap.phase2_confirmed_count(), 2);
    assert!(!snap.readiness_exited());
}

#[test]
fn ready_first_timely_triggers_quorum() {
    // Arrange
    let mut v = machine_in_phase2();
    let _ = v.apply(ready(1, TIMELY));
    let _ = v.apply(ready(2, TIMELY));
    let snap_before = v.snapshot();
    assert_eq!(snap_before.phase2_confirmed_count(), 3);
    assert!(!snap_before.readiness_exited());

    // Act
    let outcomes = match v.apply(ready(3, TIMELY)) {
        ApplyStatus::Accepted { outcomes, .. } => outcomes,
        ApplyStatus::Rejected { .. } => panic!("expected accepted"),
    };
    let snap = v.snapshot();

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::ReadyQuorumReached,
            Outcome::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(snap.phase2_confirmed_count(), 4);
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert_eq!(snap.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snap.readiness_exited());
}

#[test]
fn ready_first_delayed_triggers_quorum() {
    // Arrange
    let mut v = machine_in_phase2();
    let _ = v.apply(ready(1, TIMELY));
    let _ = v.apply(ready(2, TIMELY));
    let snap_before = v.snapshot();
    assert_eq!(snap_before.phase2_confirmed_count(), 3);
    assert!(!snap_before.readiness_exited());

    // Act
    let outcomes = match v.apply(ready(3, DELAYED)) {
        ApplyStatus::Accepted { outcomes, .. } => outcomes,
        ApplyStatus::Rejected { .. } => panic!("expected accepted"),
    };
    let snap = v.snapshot();

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::DelayedReadyAccepted { peer_id: 3 },
            Outcome::ReadyQuorumReached,
            Outcome::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(snap.phase2_confirmed_count(), 4);
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert!(snap.readiness_exited());
}

#[test]
fn local_completion_in_phase2_is_noop() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();

    // Act & Assert
    assert!(matches!(
        v.apply(Command::LocalParticipationCompleted),
        ApplyStatus::Rejected { .. }
    ));
    assert_eq!(v.snapshot(), snap_before);
}

#[test]
fn deadline_expired_exits_in_phase2() {
    // Arrange
    let mut v = machine_in_phase2();
    let snap_before = v.snapshot();
    assert_eq!(
        snap_before.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snap_before.readiness_exited());
    assert!(snap_before.local_participation_complete());

    // Act
    let outcomes = match v.apply(Command::DeadlineExpired) {
        ApplyStatus::Accepted { outcomes, .. } => outcomes,
        ApplyStatus::Rejected { .. } => panic!("expected accepted"),
    };
    let snap = v.snapshot();

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::ReadinessExited {
            mode: ReadinessExitMode::Deadline,
        }]
    );
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert_eq!(snap.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(snap.readiness_exited());
    assert!(snap.local_participation_complete());
}

#[test]
fn vibe_check_returns_correct_snapshot() {
    // Arrange & Act
    let v = machine_in_phase2();
    let snap = v.snapshot();

    // Assert
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(snap.exit_mode(), None);
    assert!(snap.local_participation_complete());
    assert!(!snap.readiness_exited());
    assert_eq!(snap.phase1_confirmed_count(), 1);
    assert_eq!(snap.phase2_confirmed_count(), 1);
    assert_eq!(snap.quorum_threshold(), 4);
}

#[test]
fn collecting_state_snapshot_inherits_correctly() {
    // Arrange
    let phase2 = ConfirmedSet::new(5);
    let (phase2, _) = phase2.confirm(1);
    let (phase2, _) = phase2.confirm(3);
    let collecting = Collecting {
        phase2,
        phase1_count: 2,
    };
    let prev = Snapshot::new(
        ReadinessLifecycleState::Phase1Active,
        Some(ReadinessExitMode::Deadline),
        false,
        true,
        99,
        99,
        4,
    );

    // Act
    let result = collecting.state_snapshot(&prev);

    // Assert
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(result.local_participation_complete());
    assert_eq!(result.phase1_confirmed_count(), 2);
    assert_eq!(result.phase2_confirmed_count(), 2);
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(result.readiness_exited());
}
