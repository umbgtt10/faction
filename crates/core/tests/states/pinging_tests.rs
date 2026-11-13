// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::answers::Answers;
use alloc::boxed::Box;
use alloc::vec;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::config::Config;
use faction::faction::Faction;
use faction::members::Members;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::state::State;
use faction::states::pinging::Pinging;

const PEER_SET: &[u64] = &[0, 1, 2, 3, 4];
const THRESHOLD: usize = 4;

extern crate alloc;

fn faction_in_pinging() -> Faction {
    let mut faction = Faction::new(
        Config::new(0, PEER_SET.to_vec(), QuorumPolicy::new(THRESHOLD)),
        Box::new(NoOpObserver),
    );
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    faction
}

fn get_collecting_peers_length(faction: &mut Faction) -> usize {
    match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view.collecting_peers().len(),
        _ => unreachable!(),
    }
}

fn get_pinging_peers_length(faction: &mut Faction) -> usize {
    match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view.pinging_peers().len(),
        _ => unreachable!(),
    }
}

#[test]
fn cluster_view_inherits_correctly() {
    // Arrange
    let pinging = Pinging::new(
        Members::new(PEER_SET.to_vec()),
        QuorumPolicy::new(THRESHOLD),
    );
    let prev = ClusterViewBuilder::default()
        .with_peer_state(PeerState::Collecting)
        .with_is_pinging_completed(true)
        .with_pinging_peers(vec![99])
        .with_collecting_peers(vec![99])
        .with_required_count(4)
        .build();

    // Act
    let result = pinging.cluster_view(&prev);

    // Assert
    assert_eq!(result.peer_state(), PeerState::Pinging);
    assert_eq!(result.pinging_peers().len(), 0);
    assert_eq!(result.collecting_peers().len(), 0);
    assert_ne!(result.peer_state(), PeerState::Bootstrapped);
    assert!(result.is_pinging_completed());
    assert_ne!(result.peer_state(), PeerState::Bootstrapped);
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
        vec![Outcome::DeadlineMissed { confirmed_count: 0 }]
    );
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
fn process_addition_committed_admits_a_peer_whose_participation_then_counts() {
    // Arrange
    let mut faction = faction_in_pinging();
    let _ = faction.process(Command::AdditionCommitted { peer_id: 9 });

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::ParticipationObserved { peer_id: 9 }));

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::ParticipationAccepted { peer_id: 9 }]
    );
    assert_eq!(cluster_view.pinging_peers(), &[1, 9]);
}

#[test]
fn process_deadline_expired_stays_receptive_in_pinging() {
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
        vec![Outcome::DeadlineMissed { confirmed_count: 0 }]
    );
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_ne!(snap.peer_state(), PeerState::Bootstrapped);
    assert_ne!(snap.peer_state(), PeerState::Bootstrapped);
    assert!(snap.deadline_missed());
    assert_eq!(snap.peer_state(), PeerState::TimedOut);
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
    assert_ne!(snap.peer_state(), PeerState::Bootstrapped);
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
    assert_eq!(snap.peer_state(), PeerState::Bootstrapped);
    assert_eq!(snap.peer_state(), PeerState::Bootstrapped);
}

#[test]
fn process_participation_observed_duplicate() {
    // Arrange
    let mut faction = faction_in_pinging();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 2 });
    let before = get_pinging_peers_length(&mut faction);

    // Act & Assert
    let outcomes = match faction.process(Command::ParticipationObserved { peer_id: 2 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateParticipationIgnored { peer_id: 2 }]
    );
    assert_eq!(get_pinging_peers_length(&mut faction), before);
}

#[test]
fn process_participation_observed_first_timely() {
    // Arrange
    let mut faction = faction_in_pinging();
    let before = get_pinging_peers_length(&mut faction);

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
    assert_eq!(get_pinging_peers_length(&mut faction), before + 1);
}

#[test]
fn process_participation_observed_non_member() {
    // Arrange
    let mut faction = faction_in_pinging();
    let before = get_pinging_peers_length(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved { peer_id: 99 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(get_pinging_peers_length(&mut faction), before);
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
    assert_ne!(snap.peer_state(), PeerState::Bootstrapped);
    assert!(!snap.is_pinging_completed());
    assert_ne!(snap.peer_state(), PeerState::Bootstrapped);
    assert_eq!(snap.pinging_peers().len(), 1);
    assert_eq!(snap.collecting_peers().len(), 0);
    assert_eq!(snap.required_count(), THRESHOLD);
}

#[test]
fn process_ready_observed_duplicate() {
    // Arrange
    let mut faction = faction_in_pinging();
    let _ = faction.process(Command::ReadyObserved { peer_id: 2 });
    let before = get_collecting_peers_length(&mut faction);

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
    assert_eq!(get_collecting_peers_length(&mut faction), before);
}

#[test]
fn process_ready_observed_first_timely() {
    // Arrange
    let mut faction = faction_in_pinging();
    let before = get_collecting_peers_length(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ReadyObserved { peer_id: 3 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::ReadyAccepted { peer_id: 3 }]);
    assert_eq!(get_collecting_peers_length(&mut faction), before + 1);
}

#[test]
fn process_ready_observed_non_member() {
    // Arrange
    let mut faction = faction_in_pinging();
    let before = get_collecting_peers_length(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ReadyObserved { peer_id: 99 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(get_collecting_peers_length(&mut faction), before);
}

#[test]
fn process_required_count_changed_is_the_count_the_node_s_completion_checks() {
    // Arrange
    let mut faction = faction_in_pinging();
    let _ = faction.process(Command::RequiredCountChanged { required_count: 1 });

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::LocalParticipationCompleted));

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
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
}
