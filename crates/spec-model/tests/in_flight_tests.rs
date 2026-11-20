// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::in_flight::InFlight;
use faction_spec_model::node_state::NodeState;
use faction_spec_model::reconfiguration::Reconfiguration;
use faction_spec_model::vocabulary::stage::Stage;

fn adding(peer_id: &str) -> Reconfiguration {
    Reconfiguration::AdditionRequested(String::from(peer_id))
}

fn removing(peer_id: &str) -> Reconfiguration {
    Reconfiguration::RemovalRequested(String::from(peer_id), None)
}

// Concluded with an addition of 4 and a removal of 2 in flight.
fn two() -> NodeState {
    NodeState {
        reconfigurations: vec![adding("4"), removing("2")],
        ..NodeStates::of(Stage::Reconfiguring, &[], &["1"])
    }
}

#[test]
fn acknowledged_of_a_peer_whose_addition_is_requested_acknowledges_it_alone() {
    // Arrange
    let state = two();

    // Act
    let changes = InFlight::acknowledged(&state, "4");

    // Assert
    assert_eq!(
        changes,
        vec![
            Reconfiguration::AdditionAcknowledged(String::from("4")),
            removing("2")
        ]
    );
}

#[test]
fn ended_of_a_peer_drops_its_change_alone() {
    // Arrange
    let state = two();

    // Act
    let changes = InFlight::ended(&state, "4");

    // Assert
    assert_eq!(changes, vec![removing("2")]);
}

#[test]
fn leaves_of_a_peer_whose_removal_stated_a_count_is_that_count() {
    // Arrange
    let state = NodeState {
        reconfigurations: vec![
            adding("4"),
            Reconfiguration::RemovalRequested(String::from("2"), Some(1)),
        ],
        ..NodeStates::of(Stage::Reconfiguring, &[], &["1"])
    };

    // Act
    let leaves = [
        InFlight::leaves(&state, Some(&String::from("2"))),
        InFlight::leaves(&two(), Some(&String::from("2"))),
        InFlight::leaves(&state, Some(&String::from("4"))),
    ];

    // Assert
    assert_eq!(leaves, [Some(1), None, None]);
}

#[test]
fn pending_of_a_change_requests_it_last() {
    // Arrange
    let state = two();

    // Act
    let changes = InFlight::pending(&state, adding("5"));

    // Assert
    assert_eq!(changes, vec![adding("4"), removing("2"), adding("5")]);
}
