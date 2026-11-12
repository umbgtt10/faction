// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::config::Config;
use faction::members::Members;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;
use faction::stage::Stage;
use faction::state::State;
use faction::states::bootstrapped::Bootstrapped;
use faction::states::collecting::Collecting;
use faction::states::initial::Initial;
use faction::states::pinging::Pinging;

fn bootstrapped() -> Stage {
    Stage::Bootstrapped(Bootstrapped::new(members(), vec![1], vec![0, 1, 2, 3]))
}

fn collecting() -> Stage {
    Stage::Collecting(Collecting::new(members(), vec![0], vec![1], false))
}

fn config() -> Config {
    Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4))
}

fn every_command() -> Vec<Command> {
    vec![
        Command::ParticipationObserved { peer_id: 0 },
        Command::ReadyObserved { peer_id: 0 },
        Command::LocalParticipationCompleted,
        Command::DeadlineExpired,
        Command::JoinRequested { peer_id: 0 },
        Command::JoinApproved { peer_id: 0 },
        Command::JoinRejected { peer_id: 0 },
        Command::Probe,
    ]
}

extern crate alloc;

fn initial() -> Stage {
    Stage::Initial(Initial::new(members()))
}

fn members() -> Members {
    Members::new(vec![0, 1, 2, 3, 4])
}

fn pinging() -> Stage {
    Stage::Pinging(Pinging::new(members()))
}

#[test]
fn admissible_commands_bootstrapped_returns_participation_joins_and_probe() {
    // Arrange
    let stage = bootstrapped();

    // Act
    let admissible = stage.admissible_commands(&config());

    // Assert
    assert_eq!(
        admissible,
        vec![
            Command::ParticipationObserved { peer_id: 0 },
            Command::JoinRequested { peer_id: 0 },
            Command::JoinApproved { peer_id: 0 },
            Command::JoinRejected { peer_id: 0 },
            Command::Probe,
        ]
    );
}

#[test]
fn admissible_commands_collecting_returns_all_but_local_participation_completed() {
    // Arrange
    let stage = collecting();

    // Act
    let admissible = stage.admissible_commands(&config());

    // Assert
    assert_eq!(
        admissible,
        vec![
            Command::ParticipationObserved { peer_id: 0 },
            Command::ReadyObserved { peer_id: 0 },
            Command::DeadlineExpired,
            Command::JoinRequested { peer_id: 0 },
            Command::JoinApproved { peer_id: 0 },
            Command::JoinRejected { peer_id: 0 },
            Command::Probe,
        ]
    );
}

#[test]
fn admissible_commands_initial_returns_all_but_deadline_expired() {
    // Arrange
    let stage = initial();

    // Act
    let admissible = stage.admissible_commands(&config());

    // Assert
    assert_eq!(
        admissible,
        vec![
            Command::ParticipationObserved { peer_id: 0 },
            Command::ReadyObserved { peer_id: 0 },
            Command::LocalParticipationCompleted,
            Command::JoinRequested { peer_id: 0 },
            Command::JoinApproved { peer_id: 0 },
            Command::JoinRejected { peer_id: 0 },
            Command::Probe,
        ]
    );
}

#[test]
fn admissible_commands_pinging_returns_every_command() {
    // Arrange
    let stage = pinging();

    // Act
    let admissible = stage.admissible_commands(&config());

    // Assert
    assert_eq!(admissible, every_command());
}

#[test]
fn cluster_view_bootstrapped_reports_bootstrapped() {
    // Arrange
    let stage = bootstrapped();
    let previous = ClusterViewBuilder::default().with_required_count(4).build();

    // Act
    let cluster_view = stage.cluster_view(&previous);

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(cluster_view.pinging_peers(), &[1]);
    assert_eq!(cluster_view.collecting_peers(), &[0, 1, 2, 3]);
}

#[test]
fn cluster_view_collecting_reports_collecting() {
    // Arrange
    let stage = collecting();
    let previous = ClusterViewBuilder::default().with_required_count(4).build();

    // Act
    let cluster_view = stage.cluster_view(&previous);

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Collecting);
    assert_eq!(cluster_view.pinging_peers(), &[1]);
    assert_eq!(cluster_view.collecting_peers(), &[0]);
}

#[test]
fn cluster_view_initial_reports_fresh() {
    // Arrange
    let stage = initial();
    let previous = ClusterViewBuilder::default()
        .with_peer_state(PeerState::Pinging)
        .with_pinging_peers(vec![9])
        .build();

    // Act
    let cluster_view = stage.cluster_view(&previous);

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Fresh);
    assert!(cluster_view.pinging_peers().is_empty());
}

#[test]
fn cluster_view_pinging_reports_pinging() {
    // Arrange
    let stage = pinging();
    let previous = ClusterViewBuilder::default().build();

    // Act
    let cluster_view = stage.cluster_view(&previous);

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Pinging);
}

#[test]
fn step_bootstrapped_member_participation_stays_bootstrapped() {
    // Arrange
    let stage = bootstrapped();
    let config = config();

    // Act
    let (outcomes, next) = stage
        .step(Command::ParticipationObserved { peer_id: 2 }, &config)
        .expect("admitted");

    // Assert
    assert_eq!(outcomes, vec![Outcome::AcknowledgeRejoin { peer_id: 2 }]);
    assert!(matches!(next, Stage::Bootstrapped(_)));
}

#[test]
fn step_bootstrapped_ready_observed_returns_none() {
    // Arrange
    let stage = bootstrapped();

    // Act
    let answered = stage.step(Command::ReadyObserved { peer_id: 1 }, &config());

    // Assert
    assert!(answered.is_none());
}

#[test]
fn step_collecting_local_participation_completed_returns_none() {
    // Arrange
    let stage = collecting();

    // Act
    let answered = stage.step(Command::LocalParticipationCompleted, &config());

    // Assert
    assert!(answered.is_none());
}

#[test]
fn step_collecting_ready_below_quorum_stays_collecting() {
    // Arrange
    let stage = collecting();
    let config = config();

    // Act
    let (outcomes, next) = stage
        .step(Command::ReadyObserved { peer_id: 2 }, &config)
        .expect("admitted");

    // Assert
    assert_eq!(outcomes, vec![Outcome::ReadyAccepted { peer_id: 2 }]);
    assert!(matches!(next, Stage::Collecting(_)));
}

#[test]
fn step_initial_deadline_expired_returns_none() {
    // Arrange
    let stage = initial();

    // Act
    let answered = stage.step(Command::DeadlineExpired, &config());

    // Assert
    assert!(answered.is_none());
}

#[test]
fn step_initial_participation_observed_returns_pinging() {
    // Arrange
    let stage = initial();
    let config = config();

    // Act
    let (outcomes, next) = stage
        .step(Command::ParticipationObserved { peer_id: 1 }, &config)
        .expect("admitted");

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::ParticipationAccepted { peer_id: 1 }]
    );
    assert!(matches!(next, Stage::Pinging(_)));
}

#[test]
fn step_pinging_deadline_expired_returns_an_answer() {
    // Arrange
    let stage = pinging();

    // Act
    let answered = stage.step(Command::DeadlineExpired, &config());

    // Assert
    assert!(answered.is_some());
}

#[test]
fn step_pinging_local_completion_below_quorum_returns_collecting() {
    // Arrange
    let stage = pinging();
    let config = config();

    // Act
    let (outcomes, next) = stage
        .step(Command::LocalParticipationCompleted, &config)
        .expect("admitted");

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
    assert!(matches!(next, Stage::Collecting(_)));
}
