// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::answers::Answers;
use alloc::boxed::Box;
use alloc::vec;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::command_kind::CommandKind;
use faction::config::Config;
use faction::faction::Faction;
use faction::instance::instance_id::InstanceId;
use faction::members::Members;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;
use faction::reconfiguration::Reconfiguration;
use faction::state::State;
use faction::states::bootstrapped::Bootstrapped;
use faction::states::coordinating::Coordinating;
use faction::states::in_flight::InFlight;
use faction::states::reconfiguring::Reconfiguring;

extern crate alloc;

// Node 0 of members 0 to 4, requiring three, bound to three changes,
// concluded, then asked to remove member 4: coordinating, below its bound.
fn reach_coordinating() -> Faction {
    let mut faction = Faction::new(
        Config::new(
            InstanceId {
                node: 0,
                incarnation: 1,
            },
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(3),
            1,
            0,
            false,
            3,
        ),
        Box::new(NoOpObserver),
    );
    let _ = faction.process(Command::LocalParticipationCompleted).output;
    for peer_id in 1..3 {
        let _ = faction
            .process(Command::ReadyObserved { peer_id, epoch: 0 })
            .output;
    }
    let _ = faction
        .process(Command::RemovalRequested { peer_id: 4 })
        .output;
    faction
}

#[test]
fn cluster_view_reports_bootstrapped_with_its_changes_in_flight() {
    // Arrange
    let coordinating = Coordinating::new(Reconfiguring::new(
        Bootstrapped::new(
            Members::new(vec![0, 1, 2]),
            QuorumPolicy::new(2),
            vec![1],
            vec![0, 2],
        ),
        InFlight::default().with(Reconfiguration::RemovalRequested { peer_id: 2 }),
    ));

    // Act
    let cluster_view = coordinating.cluster_view(&ClusterViewBuilder::default().build());

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(
        cluster_view.reconfigurations(),
        [Reconfiguration::RemovalRequested { peer_id: 2 }]
    );
}

#[test]
fn process_addition_requested_below_the_bound_takes_it() {
    // Arrange
    let mut faction = reach_coordinating();

    // Act
    let (outcomes, cluster_view, admissible) = Answers::accepted(
        faction
            .process(Command::AdditionRequested { peer_id: 9 })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::SendMembership { peer_id: 9 }]);
    assert_eq!(
        cluster_view.reconfigurations(),
        [
            Reconfiguration::RemovalRequested { peer_id: 4 },
            Reconfiguration::AdditionRequested { peer_id: 9 }
        ]
    );
    assert!(admissible.contains(&CommandKind::RemovalRequested));
}

#[test]
fn process_of_a_request_naming_a_peer_in_flight_ignores_it_as_a_duplicate() {
    // Arrange
    let mut faction = reach_coordinating();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::AdditionRequested { peer_id: 4 })
            .output,
    );

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateChangeIgnored { peer_id: 4 }]
    );
    assert_eq!(
        cluster_view.reconfigurations(),
        [Reconfiguration::RemovalRequested { peer_id: 4 }]
    );
}

#[test]
fn process_removal_requested_the_removals_in_flight_leave_needed_is_refused() {
    // Arrange
    let mut faction = reach_coordinating();
    let _ = faction
        .process(Command::RemovalRequested { peer_id: 3 })
        .output;

    // Act
    let (outcomes, _, _) = Answers::accepted(
        faction
            .process(Command::RemovalRequested { peer_id: 2 })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::RemovalRefused { peer_id: 2 }]);
}

#[test]
fn step_of_any_other_command_is_answered_as_reconfiguring_answers_it() {
    // Arrange
    let mut faction = reach_coordinating();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::RemovalAborted { peer_id: 4 })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::RemovalWithdrawn { peer_id: 4 }]);
    assert!(cluster_view.reconfigurations().is_empty());
}
