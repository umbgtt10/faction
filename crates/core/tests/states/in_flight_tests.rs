// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::config::Config;
use faction::instance::instance_id::InstanceId;
use faction::members::Members;
use faction::outcome::Outcome;
use faction::quorum_policy::QuorumPolicy;
use faction::reconfiguration::Reconfiguration;
use faction::stage::Stage;
use faction::states::bootstrapped::Bootstrapped;
use faction::states::in_flight::InFlight;

fn adding(peer_id: u64) -> Reconfiguration {
    Reconfiguration::AdditionRequested { peer_id }
}

// Node 0 bound to two changes.
fn bound_to_two() -> Config {
    Config::new(
        InstanceId {
            node: 0,
            incarnation: 1,
        },
        vec![0, 1, 2],
        QuorumPolicy::new(1),
        1,
        0,
        false,
        2,
    )
}

fn concluded() -> Bootstrapped {
    Bootstrapped::new(
        Members::new(vec![0, 1, 2]),
        QuorumPolicy::new(1),
        vec![],
        vec![0],
    )
}

extern crate alloc;

fn removing(peer_id: u64) -> Reconfiguration {
    Reconfiguration::RemovalRequested { peer_id }
}

#[test]
fn abandoned_gives_up_each_change_in_the_order_requested() {
    // Arrange
    let in_flight = InFlight::default().with(adding(4)).with(removing(2));

    // Act
    let outcomes = in_flight.abandoned();

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::ChangeAbandoned {
                reconfiguration: adding(4)
            },
            Outcome::ChangeAbandoned {
                reconfiguration: removing(2)
            },
        ]
    );
}

#[test]
fn acknowledged_of_a_peer_whose_addition_is_requested_acknowledges_it_alone() {
    // Arrange
    let in_flight = InFlight::default().with(adding(4)).with(adding(5));

    // Act
    let acknowledged = in_flight.acknowledged(5);

    // Assert
    assert_eq!(
        acknowledged,
        InFlight::default()
            .with(adding(4))
            .with(Reconfiguration::AdditionAcknowledged { peer_id: 5 })
    );
}

#[test]
fn acknowledgement_from_behind_is_refused_and_otherwise_proposed() {
    // Arrange
    let in_flight = InFlight::default().with(adding(4));

    // Act
    let answers = [
        in_flight.acknowledgement(4, 1, 2),
        in_flight.acknowledgement(4, 2, 2),
    ];

    // Assert
    assert_eq!(
        answers[0].0,
        vec![
            Outcome::StaleEpochIgnored {
                peer_id: 4,
                epoch: 1
            },
            Outcome::SendMembership { peer_id: 4 }
        ]
    );
    assert_eq!(answers[0].1, in_flight);
    assert_eq!(answers[1].0, vec![Outcome::ProposeAddition { peer_id: 4 }]);
    assert_eq!(answers[1].1, in_flight.acknowledged(4));
}

#[test]
fn adds_and_removes_say_which_change_names_the_peer() {
    // Arrange
    let in_flight = InFlight::default().with(adding(4)).with(removing(2));

    // Act
    let named = [
        in_flight.adds(4),
        in_flight.removes(4),
        in_flight.adds(2),
        in_flight.removes(2),
    ];

    // Assert
    assert_eq!(named, [true, false, false, true]);
}

#[test]
fn answered_of_an_abort_withdraws_its_change_and_of_anything_else_is_none() {
    // Arrange
    let in_flight = InFlight::default().with(adding(4)).with(removing(2));

    // Act
    let aborted = in_flight.answered(&Command::RemovalAborted { peer_id: 2 }, &concluded(), 0);
    let probed = in_flight.answered(&Command::ProbeDue, &concluded(), 0);

    // Assert
    assert_eq!(
        aborted,
        Some((
            vec![Outcome::RemovalWithdrawn { peer_id: 2 }],
            InFlight::default().with(adding(4))
        ))
    );
    assert!(probed.is_none());
}

#[test]
fn declined_of_a_removal_the_quorum_can_spare_proposes_it_again() {
    // Arrange
    let in_flight = InFlight::default().with(removing(2));

    // Act
    let (outcomes, left) = in_flight.declined(2, &concluded());

    // Assert
    assert_eq!(outcomes, vec![Outcome::ProposeRemoval { peer_id: 2 }]);
    assert_eq!(left, in_flight);
}

#[test]
fn ended_by_a_commit_naming_a_change_in_flight_is_its_peer() {
    // Arrange
    let in_flight = InFlight::default().with(adding(4)).with(removing(2));

    // Act
    let ended = [
        in_flight.ended_by(&Command::AdditionCommitted {
            peer_id: 4,
            epoch: 1,
        }),
        in_flight.ended_by(&Command::RemovalCommitted {
            peer_id: 4,
            epoch: 1,
        }),
        in_flight.ended_by(&Command::RemovalCommitted {
            peer_id: 2,
            epoch: 1,
        }),
    ];

    // Assert
    assert_eq!(ended, [Some(4), None, Some(2)]);
}

#[test]
fn named_of_a_peer_is_its_change_in_flight_if_any() {
    // Arrange
    let in_flight = InFlight::default().with(adding(4)).with(removing(2));

    // Act
    let named = [in_flight.named(2), in_flight.named(4), in_flight.named(1)];

    // Assert
    assert_eq!(named, [Some(removing(2)), Some(adding(4)), None]);
}

#[test]
fn removals_lists_every_peer_whose_removal_is_in_flight() {
    // Arrange
    let in_flight = InFlight::default()
        .with(removing(2))
        .with(adding(4))
        .with(removing(1));

    // Act
    let removals = in_flight.removals();

    // Assert
    assert_eq!(removals, vec![2, 1]);
}

#[test]
fn shown_writes_the_changes_into_the_view_in_the_order_requested() {
    // Arrange
    let in_flight = InFlight::default().with(adding(4)).with(removing(2));

    // Act
    let view = in_flight.shown(ClusterViewBuilder::default()).build();

    // Assert
    assert_eq!(view.reconfigurations(), [adding(4), removing(2)]);
}

#[test]
fn stage_of_the_changes_is_bootstrapped_coordinating_or_reconfiguring_by_the_bound() {
    // Arrange
    let held = [
        InFlight::default(),
        InFlight::default().with(adding(4)),
        InFlight::default().with(adding(4)).with(removing(2)),
    ];

    // Act
    let stages: Vec<Stage> = held
        .into_iter()
        .map(|in_flight| in_flight.stage(concluded(), &bound_to_two()))
        .collect();

    // Assert
    assert!(matches!(stages[0], Stage::Bootstrapped(_)));
    assert!(matches!(stages[1], Stage::Coordinating(_)));
    assert!(matches!(stages[2], Stage::Reconfiguring(_)));
}

#[test]
fn with_a_change_requests_it_last() {
    // Arrange
    let in_flight = InFlight::default().with(adding(4));

    // Act
    let more = in_flight.with(removing(2));

    // Assert
    assert_eq!(more, InFlight::default().with(adding(4)).with(removing(2)));
}

#[test]
fn without_a_peer_ends_its_change_alone() {
    // Arrange
    let in_flight = InFlight::default().with(adding(4)).with(removing(2));

    // Act
    let fewer = in_flight.without(4);

    // Assert
    assert_eq!(fewer, InFlight::default().with(removing(2)));
}
