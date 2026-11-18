// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::command_kind::CommandKind;
use faction::config::Config;
use faction::instance::instance_id::InstanceId;
use faction::members::Members;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;
use faction::reconfiguration::Reconfiguration;
use faction::stage::Stage;
use faction::state::State;
use faction::states::bootstrapped::Bootstrapped;
use faction::states::collecting::Collecting;
use faction::states::coordinating::Coordinating;
use faction::states::in_flight::InFlight;
use faction::states::initial::Initial;
use faction::states::pinging::Pinging;
use faction::states::reconfiguring::Reconfiguring;
use faction::states::removed::Removed;

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
    Config::new(
        InstanceId {
            node: 0,
            incarnation: 1,
        },
        vec![0, 1, 2, 3, 4],
        QuorumPolicy::new(4),
        1,
        0,
        false,
        1,
    )
}

// The same node bound to more than its one change in flight.
fn coordinating() -> Stage {
    Stage::Coordinating(Coordinating::new(Reconfiguring::new(
        Bootstrapped::new(members(), QuorumPolicy::new(4), vec![1], vec![0, 1, 2, 3]),
        InFlight::default().with(Reconfiguration::AdditionRequested { peer_id: 9 }),
    )))
}

fn every_command() -> Vec<Command> {
    vec![
        Command::ParticipationObserved {
            peer_id: 0,
            epoch: 0,
        },
        Command::ReadyObserved {
            peer_id: 0,
            epoch: 0,
        },
        Command::LocalParticipationCompleted,
        Command::DeadlineExpired,
        Command::JoinRequested { peer_id: 0 },
        Command::JoinApproved { peer_id: 0 },
        Command::JoinRejected { peer_id: 0 },
        Command::AdditionRequested { peer_id: 0 },
        Command::MembershipAcknowledged {
            peer_id: 0,
            epoch: 0,
        },
        Command::AdditionCommitted {
            peer_id: 0,
            epoch: 0,
        },
        Command::AdditionAborted { peer_id: 0 },
        Command::RemovalRequested { peer_id: 0 },
        Command::RemovalCommitted {
            peer_id: 0,
            epoch: 0,
        },
        Command::RemovalAborted { peer_id: 0 },
        Command::ProposalDeclined {
            peer_id: 0,
            epoch: 0,
        },
        Command::RequiredCountChanged {
            required_count: 0,
            epoch: 0,
        },
        Command::MembershipReceived {
            members: vec![],
            required_count: 0,
            epoch: 0,
        },
        Command::ProbeDue,
        Command::PeerHeard { peer_id: 0 },
        Command::ProbeMissed { peer_id: 0 },
        Command::IndirectProbesMissed { peer_id: 0 },
        Command::Probe,
    ]
}

// Every command but those only a concluded node admits: a change's
// coordination, and its members' liveness.
fn every_command_but_a_concluded_node_s_own() -> Vec<CommandKind> {
    every_command()
        .iter()
        .map(Command::kind)
        .filter(|kind| {
            !matches!(
                kind,
                CommandKind::AdditionRequested
                    | CommandKind::MembershipAcknowledged
                    | CommandKind::AdditionAborted
                    | CommandKind::RemovalRequested
                    | CommandKind::RemovalAborted
                    | CommandKind::ProposalDeclined
                    | CommandKind::ProbeDue
                    | CommandKind::PeerHeard
                    | CommandKind::ProbeMissed
                    | CommandKind::IndirectProbesMissed
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
        InFlight::default().with(Reconfiguration::AdditionRequested { peer_id: 9 }),
    ))
}

#[test]
fn admissible_commands_bootstrapped_returns_participation_joins_an_addition_a_count_liveness_and_probe()
 {
    // Arrange
    let stage = bootstrapped();

    // Act
    let admissible = stage.admissible_commands(&config(), 0);

    // Assert
    assert_eq!(
        admissible,
        vec![
            CommandKind::ParticipationObserved,
            CommandKind::JoinRequested,
            CommandKind::JoinApproved,
            CommandKind::JoinRejected,
            CommandKind::AdditionRequested,
            CommandKind::AdditionCommitted,
            CommandKind::RemovalRequested,
            CommandKind::RemovalCommitted,
            CommandKind::RequiredCountChanged,
            CommandKind::MembershipReceived,
            CommandKind::ProbeDue,
            CommandKind::PeerHeard,
            CommandKind::ProbeMissed,
            CommandKind::IndirectProbesMissed,
            CommandKind::Probe,
        ]
    );
}

#[test]
fn admissible_commands_collecting_returns_all_but_its_completion_and_a_concluded_node_s_own() {
    // Arrange
    let stage = collecting();

    // Act
    let admissible = stage.admissible_commands(&config(), 0);

    // Assert
    assert_eq!(
        admissible,
        vec![
            CommandKind::ParticipationObserved,
            CommandKind::ReadyObserved,
            CommandKind::DeadlineExpired,
            CommandKind::JoinRequested,
            CommandKind::JoinApproved,
            CommandKind::JoinRejected,
            CommandKind::AdditionCommitted,
            CommandKind::RemovalCommitted,
            CommandKind::RequiredCountChanged,
            CommandKind::MembershipReceived,
            CommandKind::Probe,
        ]
    );
}

#[test]
fn admissible_commands_coordinating_returns_what_reconfiguring_admits_and_both_requests() {
    // Arrange
    let stage = coordinating();

    // Act
    let admissible = stage.admissible_commands(&config(), 0);

    // Assert
    let reconfiguring = reconfiguring().admissible_commands(&config(), 0);
    let requests = [
        CommandKind::AdditionRequested,
        CommandKind::RemovalRequested,
    ];
    assert_eq!(
        admissible
            .iter()
            .filter(|kind| !requests.contains(kind))
            .copied()
            .collect::<Vec<CommandKind>>(),
        reconfiguring
    );
    assert!(requests.iter().all(|request| admissible.contains(request)));
    assert_eq!(admissible.len(), reconfiguring.len() + 2);
}

#[test]
fn admissible_commands_initial_returns_all_but_the_deadline_and_a_concluded_node_s_own() {
    // Arrange
    let stage = initial();

    // Act
    let admissible = stage.admissible_commands(&config(), 0);

    // Assert
    assert_eq!(
        admissible,
        vec![
            CommandKind::ParticipationObserved,
            CommandKind::ReadyObserved,
            CommandKind::LocalParticipationCompleted,
            CommandKind::JoinRequested,
            CommandKind::JoinApproved,
            CommandKind::JoinRejected,
            CommandKind::AdditionCommitted,
            CommandKind::RemovalCommitted,
            CommandKind::RequiredCountChanged,
            CommandKind::MembershipReceived,
            CommandKind::Probe,
        ]
    );
}

#[test]
fn admissible_commands_pinging_returns_all_but_a_concluded_node_s_own() {
    // Arrange
    let stage = pinging();

    // Act
    let admissible = stage.admissible_commands(&config(), 0);

    // Assert
    assert_eq!(admissible, every_command_but_a_concluded_node_s_own());
}

#[test]
fn admissible_commands_reconfiguring_returns_what_a_concluded_node_admits_but_an_admission_or_a_second_request()
 {
    // Arrange
    let stage = reconfiguring();

    // Act
    let admissible = stage.admissible_commands(&config(), 0);

    // Assert
    assert_eq!(
        admissible,
        vec![
            CommandKind::ParticipationObserved,
            CommandKind::JoinRequested,
            CommandKind::JoinRejected,
            CommandKind::MembershipAcknowledged,
            CommandKind::AdditionCommitted,
            CommandKind::AdditionAborted,
            CommandKind::RemovalCommitted,
            CommandKind::RemovalAborted,
            CommandKind::ProposalDeclined,
            CommandKind::RequiredCountChanged,
            CommandKind::MembershipReceived,
            CommandKind::ProbeDue,
            CommandKind::PeerHeard,
            CommandKind::ProbeMissed,
            CommandKind::IndirectProbesMissed,
            CommandKind::Probe,
        ]
    );
}

#[test]
fn admissible_commands_removed_returns_a_membership_and_a_probe() {
    // Arrange
    let stage = Stage::Removed(Removed::new(0, members(), QuorumPolicy::new(4)));

    // Act
    let admissible = stage.admissible_commands(&config(), 0);

    // Assert
    assert_eq!(
        admissible,
        vec![CommandKind::MembershipReceived, CommandKind::Probe,]
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
    assert_eq!(
        cluster_view.reconfigurations(),
        [Reconfiguration::AdditionRequested { peer_id: 9 }]
    );
    assert_eq!(cluster_view.collecting_peers(), &[0, 1, 2, 3]);
}

#[test]
fn cluster_view_removed_reports_removed_without_the_node() {
    // Arrange
    let stage = Stage::Removed(Removed::new(0, members(), QuorumPolicy::new(4)));
    let previous = ClusterViewBuilder::default()
        .with_peer_state(PeerState::Bootstrapped)
        .with_members(members())
        .build();

    // Act
    let cluster_view = stage.cluster_view(&previous);

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Removed);
    assert_eq!(cluster_view.members().as_slice(), &[1, 2, 3, 4]);
}

#[test]
fn step_bootstrapped_member_participation_stays_bootstrapped() {
    // Arrange
    let stage = bootstrapped();
    let config = config();

    // Act
    let (outcomes, next) = stage
        .step(
            Command::ParticipationObserved {
                peer_id: 2,
                epoch: 0,
            },
            &config,
            0,
        )
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
    let answered = stage.step(
        Command::ReadyObserved {
            peer_id: 1,
            epoch: 0,
        },
        &config(),
        0,
    );

    // Assert
    assert!(answered.is_none());
}

#[test]
fn step_collecting_local_participation_completed_returns_none() {
    // Arrange
    let stage = collecting();

    // Act
    let answered = stage.step(Command::LocalParticipationCompleted, &config(), 0);

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
        .step(
            Command::ReadyObserved {
                peer_id: 2,
                epoch: 0,
            },
            &config,
            0,
        )
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
    let answered = stage.step(Command::DeadlineExpired, &config(), 0);

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
        .step(
            Command::ParticipationObserved {
                peer_id: 1,
                epoch: 0,
            },
            &config,
            0,
        )
        .expect("admitted");

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::ParticipationAccepted { peer_id: 1 }]
    );
    assert!(matches!(next, Stage::Pinging(_)));
}

#[test]
fn step_of_a_commit_a_stage_does_not_admit_is_none_whatever_its_epoch() {
    // Arrange
    let stage = Stage::Removed(Removed::new(0, members(), QuorumPolicy::new(4)));

    // Act
    let answered = stage.step(
        Command::AdditionCommitted {
            peer_id: 7,
            epoch: 1,
        },
        &config(),
        3,
    );

    // Assert
    assert!(answered.is_none());
}

#[test]
fn step_of_a_commit_the_node_holds_already_is_ignored_and_changes_nothing() {
    // Arrange
    let stage = bootstrapped();

    // Act
    let (outcomes, after) = stage
        .step(
            Command::RemovalCommitted {
                peer_id: 0,
                epoch: 2,
            },
            &config(),
            3,
        )
        .expect("a commit is admitted");

    // Assert
    assert_eq!(outcomes, vec![Outcome::CommitIgnored { epoch: 2 }]);
    assert!(matches!(after, Stage::Bootstrapped(_)));
}

#[test]
fn step_of_the_node_s_own_removal_leaves_every_other_stage_removed() {
    // Arrange
    let stages = [
        initial(),
        pinging(),
        collecting(),
        bootstrapped(),
        reconfiguring(),
    ];

    // Act
    let removed: Vec<bool> = stages
        .iter()
        .map(|stage| {
            stage
                .step(
                    Command::RemovalCommitted {
                        peer_id: 0,
                        epoch: 0,
                    },
                    &config(),
                    0,
                )
                .is_some_and(|(outcomes, next)| {
                    outcomes == vec![Outcome::MemberRemoved { peer_id: 0 }]
                        && matches!(next, Stage::Removed(_))
                })
        })
        .collect();

    // Assert
    assert_eq!(removed, vec![true; 5]);
}

#[test]
fn step_pinging_deadline_expired_returns_an_answer() {
    // Arrange
    let stage = pinging();

    // Act
    let answered = stage.step(Command::DeadlineExpired, &config(), 0);

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
        .step(Command::LocalParticipationCompleted, &config, 0)
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
        .step(
            Command::MembershipAcknowledged {
                peer_id: 9,
                epoch: 0,
            },
            &config(),
            0,
        )
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
    let answered = stage.step(Command::JoinApproved { peer_id: 8 }, &config(), 0);

    // Assert
    assert!(answered.is_none());
}

#[test]
fn step_removed_answers_a_membership_alone_and_leaves_a_probe_to_the_node() {
    // Arrange
    let stage = Stage::Removed(Removed::new(0, members(), QuorumPolicy::new(4)));

    // Act
    let answered: Vec<bool> = every_command()
        .into_iter()
        .map(|command| stage.step(command, &config(), 0).is_some())
        .collect();

    // Assert
    let membership = 16;
    let expected: Vec<bool> = (0..22).map(|index| index == membership).collect();
    assert_eq!(answered, expected);
}
