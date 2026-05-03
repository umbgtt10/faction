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

use faction::states::pinging::Pinging;

const PEER_SET: &[u64] = &[0, 1, 2, 3, 4];
const THRESHOLD: usize = 4;

fn faction_in_pinging() -> Faction {
    let mut faction = Faction::new(
        Config::new(0, PEER_SET.to_vec(), QuorumPolicy::new(THRESHOLD)),
        Box::new(NoOpObserver),
    );
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    faction
}

fn p1(faction: &mut Faction) -> usize {
    match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view.pinging_peers().len(),
        _ => unreachable!(),
    }
}

fn p2(faction: &mut Faction) -> usize {
    match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view.collecting_peers().len(),
        _ => unreachable!(),
    }
}

#[test]
fn process_accepts_participation_observed() {
    // Arrange
    let mut faction = faction_in_pinging();

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved { peer_id: 2 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::ParticipationAccepted { peer_id: 2 }]
    );
}

#[test]
fn process_accepts_ready_observed() {
    // Arrange
    let mut faction = faction_in_pinging();

    // Act
    let outcomes = match faction.process(Command::ReadyObserved { peer_id: 2 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::ReadyAccepted { peer_id: 2 }]);
}

#[test]
fn process_accepts_local_participation_completed() {
    // Arrange
    let mut faction = faction_in_pinging();

    // Act
    let outcomes = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0], Outcome::LocalParticipationCompleted);
    assert_eq!(outcomes[1], Outcome::BroadcastLocalReady);
}

#[test]
fn process_accepts_deadline_expired() {
    // Arrange
    let mut faction = faction_in_pinging();

    // Act
    let outcomes = match faction.process(Command::DeadlineExpired) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::Concluded {
            mode: Conclusion::TimedOut
        }]
    );
}

#[test]
fn process_participation_observed_non_member() {
    // Arrange
    let mut faction = faction_in_pinging();
    let before = p1(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved { peer_id: 99 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(p1(&mut faction), before);
}

#[test]
fn process_participation_observed_duplicate() {
    // Arrange
    let mut faction = faction_in_pinging();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 2 });
    let before = p1(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved { peer_id: 2 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateParticipationIgnored { peer_id: 2 }]
    );
    assert_eq!(p1(&mut faction), before);
}

#[test]
fn process_participation_observed_first_timely() {
    // Arrange
    let mut faction = faction_in_pinging();
    let before = p1(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved { peer_id: 3 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::ParticipationAccepted { peer_id: 3 }]
    );
    assert_eq!(p1(&mut faction), before + 1);
}

#[test]
fn process_ready_observed_non_member() {
    // Arrange
    let mut faction = faction_in_pinging();
    let before = p2(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ReadyObserved { peer_id: 99 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(p2(&mut faction), before);
}

#[test]
fn process_ready_observed_duplicate() {
    // Arrange
    let mut faction = faction_in_pinging();
    let _ = faction.process(Command::ReadyObserved { peer_id: 2 });
    let before = p2(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ReadyObserved { peer_id: 2 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateReadyIgnored { peer_id: 2 }]
    );
    assert_eq!(p2(&mut faction), before);
}

#[test]
fn process_ready_observed_first_timely() {
    // Arrange
    let mut faction = faction_in_pinging();
    let before = p2(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ReadyObserved { peer_id: 3 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::ReadyAccepted { peer_id: 3 }]);
    assert_eq!(p2(&mut faction), before + 1);
}

#[test]
fn process_local_completion_no_quorum() {
    // Arrange & Act
    let mut faction = faction_in_pinging();
    let outcomes = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(snap.peer_state(), PeerState::Collecting);
    assert!(snap.is_pinging_completed());
    assert!(!snap.is_exited());
}

#[test]
fn process_local_completion_triggers_quorum() {
    // Arrange
    let mut faction = Faction::new(
        Config::new(0, PEER_SET.to_vec(), QuorumPolicy::new(4)),
        Box::new(NoOpObserver),
    );
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 1 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 2 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 3 });

    // Act
    let outcomes = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert!(snap.is_exited());
    assert_eq!(snap.exit_mode(), Some(Conclusion::Bootstrapped));
}

#[test]
fn process_deadline_expired_in_pinging() {
    // Arrange
    let mut faction = faction_in_pinging();

    // Act & Assert
    let outcomes = match faction.process(Command::DeadlineExpired) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    assert_eq!(
        outcomes,
        vec![Outcome::Concluded {
            mode: Conclusion::TimedOut,
        }]
    );
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert!(snap.is_exited());
    assert_eq!(snap.exit_mode(), Some(Conclusion::TimedOut));
}

#[test]
fn process_probe_in_pinging() {
    // Arrange & Act
    let mut faction = faction_in_pinging();
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(snap.peer_state(), PeerState::Pinging);
    assert!(!snap.is_exited());
    assert!(!snap.is_pinging_completed());
    assert_eq!(snap.exit_mode(), None);
    assert_eq!(snap.pinging_peers().len(), 1);
    assert_eq!(snap.collecting_peers().len(), 0);
    assert_eq!(snap.required_count(), THRESHOLD);
}

#[test]
fn cluster_view_inherits_correctly() {
    // Arrange
    let pinging = Pinging::new();
    let prev = ClusterView::new(PeerState::Collecting, true, vec![99], vec![99], 4);

    // Act
    let result = pinging.cluster_view(&prev);

    // Assert
    assert_eq!(result.peer_state(), PeerState::Pinging);
    assert_eq!(result.pinging_peers().len(), 0);
    assert_eq!(result.collecting_peers().len(), 0);
    assert_eq!(result.exit_mode(), None);
    assert!(result.is_pinging_completed());
    assert!(!result.is_exited());
}
