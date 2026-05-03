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
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::state::State;

use faction::states::timed_out::TimedOut;

fn make_faction() -> Faction {
    Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4)),
        Box::new(NoOpObserver),
    )
}

fn reach_deadline_from_pinging() -> Faction {
    let mut faction = make_faction();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let _ = faction.process(Command::DeadlineExpired);
    faction
}

fn reach_deadline_from_collecting() -> Faction {
    let mut faction = make_faction();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let _ = faction.process(Command::LocalParticipationCompleted);
    let _ = faction.process(Command::DeadlineExpired);
    faction
}

#[test]
fn process_rejects_participation_observed() {
    // Arrange
    let mut faction = reach_deadline_from_pinging();
    let snap_before = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    match faction.process(Command::ParticipationObserved { peer_id: 2 }) {
        ProcessResult::Rejected { .. } => {}
        ProcessResult::Accepted { .. } => panic!("expected rejected"),
        ProcessResult::Probed { .. } => unreachable!(),
    };
    assert_eq!(
        match faction.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn process_rejects_ready_observed() {
    // Arrange
    let mut faction = reach_deadline_from_pinging();
    let snap_before = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    match faction.process(Command::ReadyObserved { peer_id: 2 }) {
        ProcessResult::Rejected { .. } => {}
        ProcessResult::Accepted { .. } => panic!("expected rejected"),
        ProcessResult::Probed { .. } => unreachable!(),
    };
    assert_eq!(
        match faction.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn process_rejects_local_participation_completed() {
    // Arrange
    let mut faction = reach_deadline_from_pinging();
    let snap_before = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Rejected { .. } => {}
        ProcessResult::Accepted { .. } => panic!("expected rejected"),
        ProcessResult::Probed { .. } => unreachable!(),
    };
    assert_eq!(
        match faction.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn process_rejects_deadline_expired() {
    // Arrange
    let mut faction = reach_deadline_from_pinging();
    let snap_before = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act & Assert
    match faction.process(Command::DeadlineExpired) {
        ProcessResult::Rejected { .. } => {}
        ProcessResult::Accepted { .. } => panic!("expected rejected"),
        ProcessResult::Probed { .. } => unreachable!(),
    };
    assert_eq!(
        match faction.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snap_before
    );
}

#[test]
fn process_probe_after_deadline_from_pinging() {
    // Arrange & Act
    let mut faction = reach_deadline_from_pinging();
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::TimedOut);
    assert_eq!(cluster_view.conclusion(), Some(Conclusion::TimedOut));
    assert!(cluster_view.is_concluded());
    assert!(!cluster_view.is_pinging_completed());
    assert_eq!(cluster_view.pinging_peers().len(), 1);
    assert_eq!(cluster_view.collecting_peers().len(), 0);
}

#[test]
fn process_probe_after_deadline_from_collecting() {
    // Arrange & Act
    let mut faction = reach_deadline_from_collecting();
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::TimedOut);
    assert_eq!(cluster_view.conclusion(), Some(Conclusion::TimedOut));
    assert!(cluster_view.is_concluded());
    assert!(cluster_view.is_pinging_completed());
    assert_eq!(cluster_view.pinging_peers().len(), 1);
    assert_eq!(cluster_view.collecting_peers().len(), 1);
}

#[test]
fn process_post_deadline_inputs_leave_state_unchanged() {
    // Arrange
    let mut faction = reach_deadline_from_pinging();
    let snapshot_before = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    let _ = faction.process(Command::ParticipationObserved { peer_id: 2 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 2 });
    let _ = faction.process(Command::LocalParticipationCompleted);
    let _ = faction.process(Command::DeadlineExpired);

    // Assert
    assert_eq!(
        match faction.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        },
        snapshot_before
    );
}

#[test]
fn cluster_view_inherits_local_completion_from_pinging() {
    // Arrange
    let timed_out = TimedOut::new(vec![1, 2, 3], vec![9]);
    let prev = ClusterView::new(PeerState::Pinging, false, vec![], vec![], 4);

    // Act
    let result = timed_out.cluster_view(&prev);

    // Assert
    assert_eq!(result.peer_state(), PeerState::TimedOut);
    assert_eq!(result.conclusion(), Some(Conclusion::TimedOut));
    assert!(result.is_concluded());
    assert!(!result.is_pinging_completed());
    assert_eq!(result.pinging_peers(), &[1, 2, 3]);
    assert_eq!(result.collecting_peers(), &[9]);
    assert_eq!(result.required_count(), 4);
}

#[test]
fn cluster_view_inherits_local_completion_from_collecting() {
    // Arrange
    let timed_out = TimedOut::new(vec![5, 6], vec![1, 2, 3, 4]);
    let prev = ClusterView::new(PeerState::Collecting, true, vec![], vec![], 4);

    // Act
    let result = timed_out.cluster_view(&prev);

    // Assert
    assert_eq!(result.peer_state(), PeerState::TimedOut);
    assert_eq!(result.conclusion(), Some(Conclusion::TimedOut));
    assert!(result.is_concluded());
    assert!(result.is_pinging_completed());
    assert_eq!(result.pinging_peers().len(), 2);
    assert_eq!(result.collecting_peers().len(), 4);
    assert_eq!(result.required_count(), 4);
}
