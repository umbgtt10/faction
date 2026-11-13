// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;
use faction::addition::Addition;
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
use faction::states::reconfiguring::Reconfiguring;

fn bootstrapped() -> Stage {
    Stage::Bootstrapped(Bootstrapped::new(
        members(),
        QuorumPolicy::new(4),
        vec![1],
        vec![0, 1, 2, 3],
    ))
}

fn collecting() -> Stage {
    Stage::Collecting(Collecting::new(
        members(),
        QuorumPolicy::new(4),
        vec![0],
        vec![1],
        false,
    ))
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
        Command::AdditionRequested { peer_id: 0 },
        Command::MembershipAcknowledged { peer_id: 0 },
        Command::AdditionCommitted { peer_id: 0 },
        Command::AdditionAborted { peer_id: 0 },
        Command::RequiredCountChanged { required_count: 0 },
        Command::Probe,
    ]
}

// Every command but those coordinating an addition, which only a concluded
// node admits.
fn every_command_but_an_addition_s_coordination() -> Vec<Command> {
    every_command()
        .into_iter()
        .filter(|command| {
            !matches!(
                command,
                Command::AdditionRequested { .. }
                    | Command::MembershipAcknowledged { .. }
                    | Command::AdditionAborted { .. }
            )
        })
        .collect()
}

extern crate alloc;

fn initial() -> Stage {
    Stage::Initial(Initial::new(members(), QuorumPolicy::new(4)))
}

fn members() -> Members {
    Members::new(vec![0, 1, 2, 3, 4])
}

fn pinging() -> Stage {
    Stage::Pinging(Pinging::new(members(), QuorumPolicy::new(4)))
}

fn reconfiguring() -> Stage {
    Stage::Reconfiguring(Reconfiguring::new(
        Bootstrapped::new(members(), QuorumPolicy::new(4), vec![1], vec![0, 1, 2, 3]),
        9,
    ))
}

#[test]
fn admissible_commands_bootstrapped_returns_participation_joins_an_addition_a_count_and_probe() {
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
            Command::AdditionRequested { peer_id: 0 },
            Command::AdditionCommitted { peer_id: 0 },
            Command::RequiredCountChanged { required_count: 0 },
            Command::Probe,
        ]
    );
}

#[test]
fn admissible_commands_collecting_returns_all_but_its_completion_and_an_addition_s_coordination() {
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
            Command::AdditionCommitted { peer_id: 0 },
            Command::RequiredCountChanged { required_count: 0 },
            Command::Probe,
        ]
    );
}

#[test]
fn admissible_commands_initial_returns_all_but_the_deadline_and_an_addition_s_coordination() {
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
            Command::AdditionCommitted { peer_id: 0 },
            Command::RequiredCountChanged { required_count: 0 },
            Command::Probe,
        ]
    );
}

#[test]
fn admissible_commands_pinging_returns_all_but_an_addition_s_coordination() {
    // Arrange
    let stage = pinging();

    // Act
    let admissible = stage.admissible_commands(&config());

    // Assert
    assert_eq!(admissible, every_command_but_an_addition_s_coordination());
}

#[test]
fn admissible_commands_reconfiguring_returns_what_a_concluded_node_admits_but_an_admission_or_a_second_request(
) {
    // Arrange
    let stage = reconfiguring();

    // Act
    let admissible = stage.admissible_commands(&config());

    // Assert
    assert_eq!(
        admissible,
        vec![
            Command::ParticipationObserved { peer_id: 0 },
            Command::JoinRequested { peer_id: 0 },
            Command::JoinRejected { peer_id: 0 },
            Command::MembershipAcknowledged { peer_id: 0 },
            Command::AdditionCommitted { peer_id: 0 },
            Command::AdditionAborted { peer_id: 0 },
            Command::RequiredCountChanged { required_count: 0 },
            Command::Probe,
        ]
    );
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
fn cluster_view_reconfiguring_reports_bootstrapped_with_the_addition_requested() {
    // Arrange
    let stage = reconfiguring();
    let previous = ClusterViewBuilder::default().with_required_count(4).build();

    // Act
    let cluster_view = stage.cluster_view(&previous);

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(cluster_view.addition(), Addition::Requested { peer_id: 9 });
    assert_eq!(cluster_view.collecting_peers(), &[0, 1, 2, 3]);
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

#[test]
fn step_reconfiguring_acknowledgement_of_the_newcomer_stays_reconfiguring() {
    // Arrange
    let stage = reconfiguring();

    // Act
    let (outcomes, next) = stage
        .step(Command::MembershipAcknowledged { peer_id: 9 }, &config())
        .expect("admitted");

    // Assert
    assert_eq!(outcomes, vec![Outcome::ProposeAddition { peer_id: 9 }]);
    assert!(matches!(next, Stage::Reconfiguring(_)));
}

#[test]
fn step_reconfiguring_join_approved_returns_none() {
    // Arrange
    let stage = reconfiguring();

    // Act
    let answered = stage.step(Command::JoinApproved { peer_id: 8 }, &config());

    // Assert
    assert!(answered.is_none());
}
