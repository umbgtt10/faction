// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::state::State;

use faction::states::timed_out::TimedOut;

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
    let _ = f.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = f.process(Command::DeadlineExpired);
    f
}

fn reach_deadline_from_phase2() -> Faction {
    let mut f = make_faction();
    let _ = f.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = f.process(Command::LocalParticipationCompleted);
    let _ = f.process(Command::DeadlineExpired);
    f
}

#[test]
fn deal_rejects_participation_observed() {
    // Arrange
    let mut f = reach_deadline_from_phase1();
    let snap_before = match f.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    match f.process(Command::ParticipationObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    }) {
        ProcessResult::Rejected { .. } => {}
        ProcessResult::Accepted { .. } => panic!("expected rejected"),
        ProcessResult::Probed { .. } => unreachable!(),
    };
    assert_eq!(
        match f.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn deal_rejects_ready_observed() {
    // Arrange
    let mut f = reach_deadline_from_phase1();
    let snap_before = match f.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    match f.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    }) {
        ProcessResult::Rejected { .. } => {}
        ProcessResult::Accepted { .. } => panic!("expected rejected"),
        ProcessResult::Probed { .. } => unreachable!(),
    };
    assert_eq!(
        match f.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn deal_rejects_local_participation_completed() {
    // Arrange
    let mut f = reach_deadline_from_phase1();
    let snap_before = match f.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    match f.process(Command::LocalParticipationCompleted) {
        ProcessResult::Rejected { .. } => {}
        ProcessResult::Accepted { .. } => panic!("expected rejected"),
        ProcessResult::Probed { .. } => unreachable!(),
    };
    assert_eq!(
        match f.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn deal_rejects_deadline_expired() {
    // Arrange
    let mut f = reach_deadline_from_phase1();
    let snap_before = match f.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    match f.process(Command::DeadlineExpired) {
        ProcessResult::Rejected { .. } => {}
        ProcessResult::Accepted { .. } => panic!("expected rejected"),
        ProcessResult::Probed { .. } => unreachable!(),
    };
    assert_eq!(
        match f.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn vibe_check_after_deadline_from_phase1() {
    // Arrange & Act
    let mut f = reach_deadline_from_phase1();
    let s = match f.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(s.lifecycle_state(), ReadinessLifecycleState::TimedOut);
    assert_eq!(s.exit_mode(), Some(ReadinessExitMode::TimedOut));
    assert!(s.readiness_exited());
    assert!(!s.local_participation_complete());
    assert_eq!(s.phase1_confirmed_count(), 1);
    assert_eq!(s.phase2_confirmed_count(), 0);
}

#[test]
fn vibe_check_after_deadline_from_phase2() {
    // Arrange & Act
    let mut f = reach_deadline_from_phase2();
    let s = match f.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(s.lifecycle_state(), ReadinessLifecycleState::TimedOut);
    assert_eq!(s.exit_mode(), Some(ReadinessExitMode::TimedOut));
    assert!(s.readiness_exited());
    assert!(s.local_participation_complete());
    assert_eq!(s.phase1_confirmed_count(), 1);
    assert_eq!(s.phase2_confirmed_count(), 1);
}

#[test]
fn post_deadline_inputs_leave_state_unchanged() {
    // Arrange
    let mut f = reach_deadline_from_phase1();
    let snapshot_before = match f.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    let _ = f.process(Command::ParticipationObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = f.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = f.process(Command::LocalParticipationCompleted);
    let _ = f.process(Command::DeadlineExpired);

    // Assert
    assert_eq!(
        match f.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snapshot_before
    );
}

#[test]
fn timed_out_cluster_view_inherits_local_completion_from_phase1() {
    // Arrange
    let rbd = TimedOut {
        phase1_count: 3,
        phase2_count: 1,
    };
    let prev = ClusterView::new(ReadinessLifecycleState::Phase1Active, false, 99, 99, 4);

    // Act
    let result = rbd.cluster_view(&prev);

    // Assert
    assert_eq!(result.lifecycle_state(), ReadinessLifecycleState::TimedOut);
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::TimedOut));
    assert!(result.readiness_exited());
    assert!(!result.local_participation_complete());
    assert_eq!(result.phase1_confirmed_count(), 3);
    assert_eq!(result.phase2_confirmed_count(), 1);
    assert_eq!(result.quorum_threshold(), 4);
}

#[test]
fn timed_out_cluster_view_inherits_local_completion_from_phase2() {
    // Arrange
    let rbd = TimedOut {
        phase1_count: 2,
        phase2_count: 4,
    };
    let prev = ClusterView::new(ReadinessLifecycleState::Phase2Active, true, 99, 99, 4);

    // Act
    let result = rbd.cluster_view(&prev);

    // Assert
    assert_eq!(result.lifecycle_state(), ReadinessLifecycleState::TimedOut);
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::TimedOut));
    assert!(result.readiness_exited());
    assert!(result.local_participation_complete());
    assert_eq!(result.phase1_confirmed_count(), 2);
    assert_eq!(result.phase2_confirmed_count(), 4);
    assert_eq!(result.quorum_threshold(), 4);
}
