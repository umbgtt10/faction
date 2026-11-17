// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::command::Command;
use faction_spec_model::node_state::NodeState;
use faction_spec_model::provisional::Provisional;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::outcome_name::OutcomeName;
use faction_spec_model::vocabulary::stage::Stage;
use serde_json::json;

fn naming(name: CommandName, peer_id: &str) -> Command {
    Command {
        name,
        peer_id: Some(String::from(peer_id)),
        required_count: None,
        epoch: 3,
        members: Vec::new(),
    }
}

// Node 1, its membership provisional.
fn provisional() -> NodeState {
    NodeState {
        provisional: true,
        ..NodeStates::of(Stage::Initial, &[], &[])
    }
}

#[test]
fn after_an_adopted_membership_is_not_provisional() {
    // Arrange
    let membership = Command {
        members: NodeStates::peers(&["1", "2"]),
        ..naming(CommandName::MembershipReceived, "1")
    };

    // Act
    let after = Provisional::after(
        &membership,
        &[OutcomeName::MembershipAdopted],
        &provisional(),
    );

    // Assert
    assert!(!after);
}

#[test]
fn after_another_peer_s_commit_is_still_provisional() {
    // Arrange
    let commit = naming(CommandName::AdditionCommitted, "4");

    // Act
    let after = Provisional::after(&commit, &[OutcomeName::MemberAdmitted], &provisional());

    // Assert
    assert!(after);
}

#[test]
fn after_anything_at_a_node_never_provisional_is_not_provisional() {
    // Arrange
    let commit = naming(CommandName::AdditionCommitted, "1");

    // Act
    let after = Provisional::after(
        &commit,
        &[OutcomeName::DuplicateMemberIgnored],
        &NodeStates::of(Stage::Initial, &[], &[]),
    );

    // Assert
    assert!(!after);
}

#[test]
fn after_its_own_addition_held_already_is_still_provisional() {
    // Arrange
    let commit = naming(CommandName::AdditionCommitted, "1");

    // Act
    let after = Provisional::after(&commit, &[OutcomeName::CommitIgnored], &provisional());

    // Assert
    assert!(after);
}

#[test]
fn after_its_own_removal_is_not_provisional() {
    // Arrange
    let commit = naming(CommandName::RemovalCommitted, "1");

    // Act
    let after = Provisional::after(&commit, &[OutcomeName::MemberRemoved], &provisional());

    // Assert
    assert!(!after);
}

#[test]
fn after_the_commit_of_its_own_addition_is_not_provisional() {
    // Arrange
    let commit = naming(CommandName::AdditionCommitted, "1");

    // Act
    let after = Provisional::after(
        &commit,
        &[OutcomeName::DuplicateMemberIgnored],
        &provisional(),
    );

    // Assert
    assert!(!after);
}

#[test]
fn stated_names_the_membership_provisional_only_when_it_is() {
    // Arrange
    let payload = json!({"members": ["1"]});

    // Act
    let stated = [
        Provisional::stated(payload.clone(), true),
        Provisional::stated(payload, false),
    ];

    // Assert
    assert_eq!(
        stated,
        [
            json!({"members": ["1"], "provisional": true}),
            json!({"members": ["1"]}),
        ]
    );
}
