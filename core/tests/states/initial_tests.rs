// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

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

use faction::states::initial::Initial;

fn test_faction() -> Faction {
    Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4)),
        Box::new(NoOpObserver),
    )
}

#[test]
fn process_accepts_participation_observed() {
    // Arrange
    let mut faction = test_faction();

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved { peer_id: 1 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(
        outcomes,
        vec![Outcome::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snap.peer_state(), PeerState::Pinging);
    assert_eq!(snap.pinging_peers().len(), 1);
}

#[test]
fn process_accepts_ready_observed() {
    // Arrange
    let mut faction = test_faction();

    // Act
    let outcomes = match faction.process(Command::ReadyObserved { peer_id: 1 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(outcomes, vec![Outcome::ReadyAccepted { peer_id: 1 }]);
    assert_eq!(snap.peer_state(), PeerState::Pinging);
    assert_eq!(snap.collecting_peers().len(), 1);
}

#[test]
fn process_accepts_local_participation_completed() {
    // Arrange
    let mut faction = test_faction();

    // Act
    let outcomes = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        other => panic!("expected Accepted, got {other:?}"),
    };

    // Assert
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(
        outcomes,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
    assert_eq!(snap.peer_state(), PeerState::Collecting);
    assert!(snap.is_pinging_completed());
    assert_eq!(snap.collecting_peers().len(), 1);
}

#[test]
fn process_rejects_deadline_expired() {
    // Arrange
    let mut faction = test_faction();

    // Act & Assert
    assert!(matches!(
        faction.process(Command::DeadlineExpired),
        ProcessResult::Rejected { .. }
    ));

    // Assert
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(snap.peer_state(), PeerState::Fresh);
    assert_eq!(snap.conclusion(), None);
}

#[test]
fn process_stays_in_initial_after_rejected() {
    // Arrange
    let mut faction = test_faction();

    // Act & Assert
    assert!(matches!(
        faction.process(Command::DeadlineExpired),
        ProcessResult::Rejected { .. }
    ));

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved { peer_id: 1 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(
        outcomes,
        vec![Outcome::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snap.pinging_peers().len(), 1);
}

#[test]
fn process_rejects_invalid_commands() {
    // Arrange
    let mut faction = test_faction();

    // Act & Assert
    assert!(matches!(
        faction.process(Command::DeadlineExpired),
        ProcessResult::Rejected { .. }
    ));
    assert!(matches!(
        faction.process(Command::DeadlineExpired),
        ProcessResult::Rejected { .. }
    ));

    // Assert
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(snap.peer_state(), PeerState::Fresh);
    assert_eq!(snap.pinging_peers().len(), 0);
    assert_eq!(snap.collecting_peers().len(), 0);
    assert_eq!(snap.conclusion(), None);
    assert!(!snap.is_concluded());
    assert!(!snap.is_pinging_completed());
}

#[test]
fn process_participation_non_member_from_initial() {
    // Arrange
    let mut faction = test_faction();

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved { peer_id: 99 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(snap.pinging_peers().len(), 0);
    assert_eq!(snap.peer_state(), PeerState::Pinging);
}

#[test]
fn process_ready_non_member_from_initial() {
    // Arrange
    let mut faction = test_faction();

    // Act
    let outcomes = match faction.process(Command::ReadyObserved { peer_id: 99 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(snap.collecting_peers().len(), 0);
}

#[test]
fn new_returns_initial_state() {
    // Arrange & Act
    let initial = Initial::new();
    let config = Config::new(0, vec![0], QuorumPolicy::new(1));
    let (outcomes, new_state) =
        initial.step(Command::ParticipationObserved { peer_id: 0 }, &config);

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::ParticipationAccepted { peer_id: 0 }]
    );
    assert!(
        new_state
            .as_ref()
            .cluster_view(&ClusterView::new(
                PeerState::Pinging,
                false,
                vec![0],
                vec![],
                1
            ))
            .pinging_peers()
            .len()
            > 0
    );
}

#[test]
fn process_probe_returns_fresh_state() {
    // Arrange & Act
    let mut faction = test_faction();
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(snap.peer_state(), PeerState::Fresh);
    assert_eq!(snap.conclusion(), None);
    assert!(!snap.is_pinging_completed());
    assert!(!snap.is_concluded());
    assert_eq!(snap.pinging_peers().len(), 0);
    assert_eq!(snap.collecting_peers().len(), 0);
    assert_eq!(snap.required_count(), 4);
}

#[test]
fn process_local_participation_completed_single_node_bootstraps() {
    // Arrange
    let mut faction = Faction::new(
        Config::new(0, vec![0], QuorumPolicy::new(1)),
        Box::new(NoOpObserver),
    );

    // Act
    let outcomes = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        other => panic!("expected Accepted, got {other:?}"),
    };

    // Assert
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(
        outcomes,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped
            },
        ]
    );
    assert_eq!(snap.peer_state(), PeerState::Bootstrapped);
    assert!(snap.is_concluded());
}

#[test]
fn cluster_view_inherits_correctly() {
    // Arrange
    let prev = ClusterView::new(PeerState::Collecting, true, vec![99], vec![99], 4);

    // Act
    let result = Initial.cluster_view(&prev);

    // Assert
    assert_eq!(result.peer_state(), PeerState::Fresh);
    assert_eq!(result.pinging_peers().len(), 0);
    assert_eq!(result.collecting_peers().len(), 0);
    assert_eq!(result.conclusion(), None);
    assert!(!result.is_pinging_completed());
    assert!(!result.is_concluded());
}
