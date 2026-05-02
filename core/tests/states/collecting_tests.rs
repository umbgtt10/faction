// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::state::State;

use faction::states::collecting::Collecting;

const THRESHOLD: usize = 4;

fn machine_in_collecting() -> Faction {
    let mut v = Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(THRESHOLD)),
        Box::new(NoOpObserver),
    );
    let _ = v.process(Command::ParticipationObserved { peer_id: 1 });
    let _ = v.process(Command::LocalParticipationCompleted);
    v
}

fn ready(peer_id: u64) -> Command {
    Command::ReadyObserved { peer_id }
}

#[test]
fn deal_accepts_ready_observed() {
    // Arrange & Act
    let mut v = machine_in_collecting();
    let outcomes = match v.process(ready(1)) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::ReadyAccepted { peer_id: 1 }]);
}

#[test]
fn deal_accepts_deadline_expired() {
    // Arrange & Act
    let mut v = machine_in_collecting();
    let outcomes = match v.process(Command::DeadlineExpired) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let snap = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::Concluded {
            mode: Conclusion::TimedOut,
        }]
    );
    assert_eq!(snap.exit_mode(), Some(Conclusion::TimedOut));
    assert!(snap.is_exited());
}

#[test]
fn deal_rejects_participation_observed() {
    // Arrange
    let mut v = machine_in_collecting();
    let snap_before = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    assert!(matches!(
        v.process(Command::ParticipationObserved { peer_id: 2 }),
        ProcessResult::Rejected { .. }
    ));
    assert_eq!(
        match v.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn deal_rejects_is_pinging_completedd() {
    // Arrange
    let mut v = machine_in_collecting();
    let snap_before = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    assert!(matches!(
        v.process(Command::LocalParticipationCompleted),
        ProcessResult::Rejected { .. }
    ));
    assert_eq!(
        match v.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn participation_non_member_is_noop() {
    // Arrange
    let mut v = machine_in_collecting();
    let snap_before = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    assert!(matches!(
        v.process(Command::ParticipationObserved { peer_id: 99 }),
        ProcessResult::Rejected { .. }
    ));
    assert_eq!(
        match v.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn ready_non_member_rejected() {
    // Arrange
    let mut v = machine_in_collecting();
    let snap_before = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    let outcomes = match v.process(ready(99)) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(
        match v.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn ready_duplicate_rejected() {
    // Arrange
    let mut v = machine_in_collecting();
    let _ = v.process(ready(1));
    let snap_before = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    let outcomes = match v.process(ready(1)) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(
        match v.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn ready_first_timely_no_quorum() {
    // Arrange
    let mut v = machine_in_collecting();
    let snap_before = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    assert_eq!(snap_before.collecting_peers().len(), 1);
    assert_eq!(snap_before.peer_state(), PeerState::Collecting);

    // Act
    let outcomes = match v.process(ready(1)) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let snap = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::ReadyAccepted { peer_id: 1 }]);
    assert_eq!(snap.collecting_peers().len(), 2);
    assert_eq!(snap.peer_state(), PeerState::Collecting);
    assert!(!snap.is_exited());
}

#[test]
fn ready_first_timely_triggers_quorum() {
    // Arrange
    let mut v = machine_in_collecting();
    let _ = v.process(ready(1));
    let _ = v.process(ready(2));
    let snap_before = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    assert_eq!(snap_before.collecting_peers().len(), 3);
    assert!(!snap_before.is_exited());

    // Act
    let outcomes = match v.process(ready(3)) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let snap = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    assert_eq!(snap.collecting_peers().len(), 4);
    assert_eq!(snap.peer_state(), PeerState::Bootstrapped);
    assert_eq!(snap.exit_mode(), Some(Conclusion::Bootstrapped));
    assert!(snap.is_exited());
}

#[test]
fn local_completion_in_collecting_is_noop() {
    // Arrange
    let mut v = machine_in_collecting();
    let snap_before = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    assert!(matches!(
        v.process(Command::LocalParticipationCompleted),
        ProcessResult::Rejected { .. }
    ));
    assert_eq!(
        match v.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn deadline_expired_exits_in_collecting() {
    // Arrange
    let mut v = machine_in_collecting();
    let snap_before = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    assert_eq!(snap_before.peer_state(), PeerState::Collecting);
    assert!(!snap_before.is_exited());
    assert!(snap_before.is_pinging_completed());

    // Act
    let outcomes = match v.process(Command::DeadlineExpired) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let snap = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::Concluded {
            mode: Conclusion::TimedOut,
        }]
    );
    assert_eq!(snap.peer_state(), PeerState::TimedOut);
    assert_eq!(snap.exit_mode(), Some(Conclusion::TimedOut));
    assert!(snap.is_exited());
    assert!(snap.is_pinging_completed());
}

#[test]
fn vibe_check_returns_correct_snapshot() {
    // Arrange & Act
    let mut v = machine_in_collecting();
    let snap = match v.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(snap.peer_state(), PeerState::Collecting);
    assert_eq!(snap.exit_mode(), None);
    assert!(snap.is_pinging_completed());
    assert!(!snap.is_exited());
    assert_eq!(snap.pinging_peers().len(), 1);
    assert_eq!(snap.collecting_peers().len(), 1);
    assert_eq!(snap.required_count(), 4);
}

#[test]
fn collecting_cluster_view_inherits_correctly() {
    // Arrange
    let collecting_set = vec![1, 3];
    let collecting = Collecting::new(collecting_set, vec![5, 6]);
    let prev = ClusterView::new(PeerState::Pinging, false, vec![], vec![], 4);

    // Act
    let result = collecting.cluster_view(&prev);

    // Assert
    assert_eq!(result.peer_state(), PeerState::Collecting);
    assert!(result.is_pinging_completed());
    assert_eq!(result.pinging_peers().len(), 2);
    assert_eq!(result.collecting_peers(), &[1, 3]);
    assert_eq!(result.exit_mode(), None);
    assert!(!result.is_exited());
}
