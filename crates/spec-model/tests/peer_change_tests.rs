// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::command::Command;
use faction_spec_model::node_state::NodeState;
use faction_spec_model::peer_change::PeerChange;
use faction_spec_model::reconfiguration::Reconfiguration;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::stage::Stage;

fn acknowledged(peer_id: &str) -> Reconfiguration {
    Reconfiguration::AdditionAcknowledged(String::from(peer_id))
}

fn naming(name: CommandName, peer_id: &str) -> Command {
    Command {
        name,
        peer_id: Some(String::from(peer_id)),
        required_count: None,
        leaves: None,
        epoch: 0,
        members: Vec::new(),
    }
}

fn removal(peer_id: &str) -> Reconfiguration {
    Reconfiguration::RemovalRequested(String::from(peer_id), None)
}

fn requested(peer_id: &str) -> Reconfiguration {
    Reconfiguration::AdditionRequested(String::from(peer_id))
}

// Concluded with the addition of 4 requested, the addition of 5 acknowledged
// and the removal of 2 requested.
fn three() -> NodeState {
    NodeState {
        reconfigurations: vec![requested("4"), acknowledged("5"), removal("2")],
        changes_in_flight: 3,
        ..NodeStates::of(Stage::Reconfiguring, &[], &["1"])
    }
}

#[test]
fn awaits_acknowledgement_of_a_peer_named_by_no_change_is_false() {
    // Arrange
    let state = three();
    let acknowledgement = naming(CommandName::MembershipAcknowledged, "3");

    // Act
    let awaits = PeerChange::awaits_acknowledgement(&acknowledgement, &state);

    // Assert
    assert!(!awaits);
}

#[test]
fn awaits_acknowledgement_of_a_peer_whose_addition_is_acknowledged_or_removal_requested_is_false() {
    // Arrange
    let state = three();

    // Act
    let awaits = [
        PeerChange::awaits_acknowledgement(
            &naming(CommandName::MembershipAcknowledged, "5"),
            &state,
        ),
        PeerChange::awaits_acknowledgement(
            &naming(CommandName::MembershipAcknowledged, "2"),
            &state,
        ),
    ];

    // Assert
    assert_eq!(awaits, [false, false]);
}

#[test]
fn awaits_acknowledgement_of_the_peer_whose_addition_is_requested_is_true() {
    // Arrange
    let state = three();
    let acknowledgement = naming(CommandName::MembershipAcknowledged, "4");

    // Act
    let awaits = PeerChange::awaits_acknowledgement(&acknowledgement, &state);

    // Assert
    assert!(awaits);
}

#[test]
fn awaits_decline_of_a_peer_named_by_no_change_is_false() {
    // Arrange
    let state = three();
    let decline = naming(CommandName::ProposalDeclined, "3");

    // Act
    let awaits = PeerChange::awaits_decline(&decline, &state);

    // Assert
    assert!(!awaits);
}

#[test]
fn awaits_decline_of_a_peer_whose_addition_is_acknowledged_or_removal_requested_is_true() {
    // Arrange
    let state = three();

    // Act
    let awaits = [
        PeerChange::awaits_decline(&naming(CommandName::ProposalDeclined, "5"), &state),
        PeerChange::awaits_decline(&naming(CommandName::ProposalDeclined, "2"), &state),
    ];

    // Assert
    assert_eq!(awaits, [true, true]);
}

#[test]
fn awaits_decline_of_the_peer_whose_addition_is_requested_is_false() {
    // Arrange
    let state = three();
    let decline = naming(CommandName::ProposalDeclined, "4");

    // Act
    let awaits = PeerChange::awaits_decline(&decline, &state);

    // Assert
    assert!(!awaits);
}

#[test]
fn in_flight_of_a_command_naming_no_peer_is_false() {
    // Arrange
    let state = three();
    let completion = Command {
        peer_id: None,
        ..naming(CommandName::LocalParticipationCompleted, "4")
    };

    // Act
    let in_flight = PeerChange::in_flight(&completion, &state);

    // Assert
    assert!(!in_flight);
}

#[test]
fn in_flight_of_a_peer_named_by_a_change_of_each_kind_is_true() {
    // Arrange
    let state = three();

    // Act
    let in_flight = [
        PeerChange::in_flight(&naming(CommandName::AdditionRequested, "4"), &state),
        PeerChange::in_flight(&naming(CommandName::AdditionRequested, "5"), &state),
        PeerChange::in_flight(&naming(CommandName::RemovalRequested, "2"), &state),
    ];

    // Assert
    assert_eq!(in_flight, [true, true, true]);
}

#[test]
fn in_flight_of_a_peer_named_by_no_change_is_false() {
    // Arrange
    let state = three();
    let request = naming(CommandName::RemovalRequested, "3");

    // Act
    let in_flight = PeerChange::in_flight(&request, &state);

    // Assert
    assert!(!in_flight);
}

#[test]
fn pending_of_a_command_naming_a_peer_named_by_no_change_is_false() {
    // Arrange
    let state = three();

    // Act
    let pending = [
        PeerChange::pending(&naming(CommandName::AdditionCommitted, "3"), &state),
        PeerChange::pending(&naming(CommandName::RemovalCommitted, "3"), &state),
    ];

    // Assert
    assert_eq!(pending, [false, false]);
}

#[test]
fn pending_of_a_removal_s_command_naming_a_peer_whose_addition_is_in_flight_is_false() {
    // Arrange
    let state = three();

    // Act
    let pending = [
        PeerChange::pending(&naming(CommandName::RemovalCommitted, "4"), &state),
        PeerChange::pending(&naming(CommandName::RemovalAborted, "5"), &state),
    ];

    // Assert
    assert_eq!(pending, [false, false]);
}

#[test]
fn pending_of_a_removal_s_command_naming_the_peer_of_the_removal_in_flight_is_true() {
    // Arrange
    let state = three();

    // Act
    let pending = [
        PeerChange::pending(&naming(CommandName::RemovalCommitted, "2"), &state),
        PeerChange::pending(&naming(CommandName::RemovalAborted, "2"), &state),
    ];

    // Assert
    assert_eq!(pending, [true, true]);
}

#[test]
fn pending_of_an_addition_s_command_naming_a_peer_whose_addition_is_in_flight_is_true() {
    // Arrange
    let state = three();

    // Act
    let pending = [
        PeerChange::pending(&naming(CommandName::AdditionCommitted, "4"), &state),
        PeerChange::pending(&naming(CommandName::AdditionAborted, "5"), &state),
    ];

    // Assert
    assert_eq!(pending, [true, true]);
}

#[test]
fn pending_of_an_addition_s_command_naming_the_peer_of_the_removal_in_flight_is_false() {
    // Arrange
    let state = three();
    let commit = naming(CommandName::AdditionCommitted, "2");

    // Act
    let pending = PeerChange::pending(&commit, &state);

    // Assert
    assert!(!pending);
}
