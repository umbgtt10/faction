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
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::node_state::NodeState;
use faction::state::State;

use faction::states::initial::Initial;

fn test_machine() -> Faction {
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

#[test]
fn deal_accepts_participation_observed() {
    // Arrange
    let mut faction = test_machine();

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    }) {
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
    assert_eq!(
        snap.node_state(),
        NodeState::Pinging
    );
    assert_eq!(snap.pinging_confirmed_count(), 1);
}

#[test]
fn deal_accepts_ready_observed() {
    // Arrange
    let mut faction = test_machine();

    // Act
    let outcomes = match faction.process(Command::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    }) {
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
    assert_eq!(
        snap.node_state(),
        NodeState::Pinging
    );
    assert_eq!(snap.collecting_confirmed_count(), 1);
}

#[test]
fn deal_rejects_local_participation_completed() {
    // Arrange
    let mut faction = test_machine();

    // Act & Assert
    assert!(matches!(
        faction.process(Command::LocalParticipationCompleted),
        ProcessResult::Rejected { .. }
    ));

    // Assert
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(
        snap.node_state(),
        NodeState::Pinging
    );
    assert_eq!(snap.collecting_confirmed_count(), 0);
}

#[test]
fn deal_rejects_deadline_expired() {
    // Arrange
    let mut faction = test_machine();

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
    assert_eq!(
        snap.node_state(),
        NodeState::Pinging
    );
    assert_eq!(snap.exit_mode(), None);
}

#[test]
fn stays_in_initial_after_rejected_input() {
    // Arrange
    let mut faction = test_machine();

    // Act & Assert
    assert!(matches!(
        faction.process(Command::DeadlineExpired),
        ProcessResult::Rejected { .. }
    ));

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    }) {
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
    assert_eq!(snap.pinging_confirmed_count(), 1);
}

#[test]
fn multiple_rejected_inputs_keep_initial_unchanged() {
    // Arrange
    let mut faction = test_machine();

    // Act & Assert
    assert!(matches!(
        faction.process(Command::LocalParticipationCompleted),
        ProcessResult::Rejected { .. }
    ));
    assert!(matches!(
        faction.process(Command::DeadlineExpired),
        ProcessResult::Rejected { .. }
    ));
    assert!(matches!(
        faction.process(Command::LocalParticipationCompleted),
        ProcessResult::Rejected { .. }
    ));

    // Assert
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(
        snap.node_state(),
        NodeState::Pinging
    );
    assert_eq!(snap.pinging_confirmed_count(), 0);
    assert_eq!(snap.collecting_confirmed_count(), 0);
    assert_eq!(snap.exit_mode(), None);
    assert!(!snap.readiness_exited());
    assert!(!snap.local_participation_complete());
}

#[test]
fn punch_participation_non_member_from_initial() {
    // Arrange
    let mut faction = test_machine();

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    }) {
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
    assert_eq!(snap.pinging_confirmed_count(), 0);
    assert_eq!(
        snap.node_state(),
        NodeState::Pinging
    );
}

#[test]
fn punch_participation_delayed_from_initial() {
    // Arrange
    let mut faction = test_machine();

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 8,
        current_marker: 10,
    }) {
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
        vec![Outcome::DelayedParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snap.pinging_confirmed_count(), 1);
}

#[test]
fn punch_ready_non_member_from_initial() {
    // Arrange
    let mut faction = test_machine();

    // Act
    let outcomes = match faction.process(Command::ReadyObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    }) {
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
    assert_eq!(snap.collecting_confirmed_count(), 0);
}

#[test]
fn vibe_check_returns_phase1_active_with_zeros() {
    // Arrange & Act
    let mut faction = test_machine();
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(
        snap.node_state(),
        NodeState::Pinging
    );
    assert_eq!(snap.exit_mode(), None);
    assert!(!snap.local_participation_complete());
    assert!(!snap.readiness_exited());
    assert_eq!(snap.pinging_confirmed_count(), 0);
    assert_eq!(snap.collecting_confirmed_count(), 0);
    assert_eq!(snap.quorum_threshold(), 4);
}

#[test]
fn initial_cluster_view_inherits_correctly() {
    // Arrange
    let prev = ClusterView::new(NodeState::Collecting, true, 99, 99, 4);

    // Act
    let result = Initial.cluster_view(&prev);

    // Assert
    assert_eq!(
        result.node_state(),
        NodeState::Pinging
    );
    assert_eq!(result.pinging_confirmed_count(), 0);
    assert_eq!(result.collecting_confirmed_count(), 0);
    assert_eq!(result.exit_mode(), None);
    assert!(result.local_participation_complete());
    assert!(!result.readiness_exited());
}
