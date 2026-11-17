// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::config::Config;
use faction::instance::instance_id::InstanceId;
use faction::members::Members;
use faction::miss_count::MissCount;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::quorum_policy::QuorumPolicy;
use faction::reconfiguration::Reconfiguration;
use faction::stage::Stage;
use faction::state::State;
use faction::states::removed::Removed;

fn config() -> Config {
    Config::new(
        InstanceId {
            node: 0,
            incarnation: 1,
        },
        vec![0, 1, 2],
        QuorumPolicy::new(2),
        1,
        0,
        false,
    )
}

extern crate alloc;

#[test]
fn cluster_view_shows_what_the_node_held_but_itself_and_its_liveness() {
    // Arrange
    let previous = ClusterViewBuilder::default()
        .with_peer_state(PeerState::Bootstrapped)
        .with_is_pinging_completed(true)
        .with_pinging_peers(vec![0, 1])
        .with_collecting_peers(vec![1, 0])
        .with_required_count(2)
        .with_deadline_missed(true)
        .with_members(Members::new(vec![0, 1, 2]))
        .with_reconfiguration(Reconfiguration::RemovalRequested { peer_id: 0 })
        .with_suspected(vec![2])
        .with_misses(vec![MissCount {
            peer_id: 1,
            misses: 1,
        }])
        .with_last_probed(Some(1))
        .with_quorum(Quorum::Held)
        .build();

    // Act
    let cluster_view =
        Removed::new(0, Members::new(vec![0, 1, 2]), QuorumPolicy::new(2)).cluster_view(&previous);

    // Assert
    assert_eq!(
        cluster_view,
        ClusterViewBuilder::default()
            .with_peer_state(PeerState::Removed)
            .with_is_pinging_completed(true)
            .with_pinging_peers(vec![1])
            .with_collecting_peers(vec![1])
            .with_required_count(2)
            .with_deadline_missed(true)
            .with_members(Members::new(vec![1, 2]))
            .build()
    );
    assert_eq!(cluster_view.peer_state(), PeerState::Removed);
}

#[test]
fn departure_of_a_membership_not_naming_the_node_removes_it() {
    // Arrange
    let membership = Command::MembershipReceived {
        members: vec![1, 2],
        required_count: 1,
        epoch: 2,
    };

    // Act
    let departure = Removed::departure(
        &membership,
        0,
        &Members::new(vec![0, 1, 2]),
        QuorumPolicy::new(2),
    );

    // Assert
    let (outcomes, stage) = departure.expect("the membership does not name the node");
    assert_eq!(
        outcomes,
        vec![
            Outcome::MembershipAdopted { epoch: 2 },
            Outcome::MemberRemoved { peer_id: 0 },
        ]
    );
    assert!(matches!(stage, Stage::Removed(_)));
}

#[test]
fn departure_of_anything_else_is_none() {
    // Arrange
    let commands = [
        Command::RemovalCommitted {
            peer_id: 1,
            epoch: 0,
        },
        Command::MembershipReceived {
            members: vec![0, 1],
            required_count: 1,
            epoch: 2,
        },
        Command::Probe,
    ];

    // Act
    let departures = commands.map(|command| {
        Removed::departure(
            &command,
            0,
            &Members::new(vec![0, 1, 2]),
            QuorumPolicy::new(2),
        )
        .is_none()
    });

    // Assert
    assert_eq!(departures, [true; 3]);
}

#[test]
fn own_removal_answers_the_node_removed_and_leaves_it_removed() {
    // Arrange & Act
    let (outcomes, stage) =
        Removed::own_removal(0, &Members::new(vec![0, 1, 2]), QuorumPolicy::new(2));

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberRemoved { peer_id: 0 }]);
    assert!(matches!(stage, Stage::Removed(_)));
}

#[test]
fn step_of_a_membership_naming_the_node_takes_it_back_to_initial() {
    // Arrange
    let removed = Removed::new(0, Members::new(vec![1, 2]), QuorumPolicy::new(2));

    // Act
    let (outcomes, stage) = removed
        .step(
            Command::MembershipReceived {
                members: vec![1, 2, 0],
                required_count: 3,
                epoch: 4,
            },
            &config(),
            1,
        )
        .expect("a membership is admitted");

    // Assert
    assert_eq!(outcomes, vec![Outcome::MembershipAdopted { epoch: 4 }]);
    assert!(matches!(stage, Stage::Initial(_)));
}

#[test]
fn step_of_a_membership_not_naming_the_node_leaves_it_removed_holding_it() {
    // Arrange
    let removed = Removed::new(0, Members::new(vec![1, 2]), QuorumPolicy::new(2));
    let previous = ClusterViewBuilder::default()
        .with_collecting_peers(vec![1, 2])
        .build();

    // Act
    let (outcomes, stage) = removed
        .step(
            Command::MembershipReceived {
                members: vec![2, 3],
                required_count: 1,
                epoch: 4,
            },
            &config(),
            1,
        )
        .expect("a membership is admitted");

    // Assert
    let view = stage.cluster_view(&previous);
    assert_eq!(outcomes, vec![Outcome::MembershipAdopted { epoch: 4 }]);
    assert_eq!(view.peer_state(), PeerState::Removed);
    assert_eq!(view.members(), &Members::new(vec![2, 3]));
    assert_eq!(view.collecting_peers(), &[2]);
    assert_eq!(view.required_count(), 1);
}

#[test]
fn step_of_any_command_is_none() {
    // Arrange
    let removed = Removed::new(0, Members::new(vec![0, 1, 2]), QuorumPolicy::new(2));

    // Act
    let answered = [
        Command::RemovalCommitted {
            peer_id: 0,
            epoch: 0,
        },
        Command::ProbeDue,
        Command::JoinApproved { peer_id: 3 },
        Command::Probe,
    ]
    .map(|command| removed.step(command, &config(), 0).is_none());

    // Assert
    assert_eq!(answered, [true; 4]);
}
