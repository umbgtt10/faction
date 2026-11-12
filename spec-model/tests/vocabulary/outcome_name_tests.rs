// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::command::Command;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::outcome_name::OutcomeName;
use faction_spec_model::vocabulary::stage::Stage;
use serde_json::json;

fn signal_from(peer_id: &str) -> Command {
    Command {
        name: CommandName::ReadyObserved,
        peer_id: Some(String::from(peer_id)),
    }
}

#[test]
fn encoded_of_an_outcome_carrying_nothing_is_a_unit() {
    // Arrange
    let completion = Command {
        name: CommandName::LocalParticipationCompleted,
        peer_id: None,
    };

    // Act
    let encoded = OutcomeName::BroadcastLocalReady
        .encoded(&completion, &NodeStates::of(Stage::Pinging, &[], &[]));

    // Assert
    assert_eq!(encoded, json!({"BroadcastLocalReady": {}}));
}

#[test]
fn encoded_of_an_outcome_naming_a_peer_names_the_command_s_peer() {
    // Arrange & Act
    let encoded = OutcomeName::NonMemberIgnored
        .encoded(&signal_from("9"), &NodeStates::of(Stage::Pinging, &[], &[]));

    // Assert
    assert_eq!(encoded, json!({"NonMemberIgnored": {"peer_id": "9"}}));
}

#[test]
fn encoded_of_concluded_names_the_mode_bootstrapped() {
    // Arrange & Act
    let encoded = OutcomeName::Concluded.encoded(
        &signal_from("2"),
        &NodeStates::of(Stage::Collecting, &[], &["1"]),
    );

    // Assert
    assert_eq!(
        encoded,
        json!({"Concluded": {"mode": {"Bootstrapped": {}}}})
    );
}

#[test]
fn encoded_of_deadline_missed_counts_the_collecting_peers() {
    // Arrange
    let expiry = Command {
        name: CommandName::DeadlineExpired,
        peer_id: None,
    };

    // Act
    let encoded = OutcomeName::DeadlineMissed.encoded(
        &expiry,
        &NodeStates::of(Stage::Collecting, &[], &["3", "1"]),
    );

    // Assert
    assert_eq!(encoded, json!({"DeadlineMissed": {"confirmed_count": "2"}}));
}

#[test]
fn name_of_each_outcome_is_the_vocabulary_s() {
    // Arrange & Act
    let names: Vec<&str> = OutcomeName::ALL.iter().map(|name| name.name()).collect();

    // Assert
    assert_eq!(
        names,
        [
            "ParticipationAccepted",
            "ReadyAccepted",
            "DuplicateParticipationIgnored",
            "DuplicateReadyIgnored",
            "NonMemberIgnored",
            "LocalParticipationCompleted",
            "BroadcastLocalReady",
            "AcknowledgeRejoin",
            "DeadlineMissed",
            "EmitJoinRequest",
            "MemberAdmitted",
            "DuplicateMemberIgnored",
            "JoinDenied",
            "Concluded",
        ]
    );
}

#[test]
fn of_each_outcome_s_name_reads_that_outcome() {
    // Arrange & Act
    let read: Vec<OutcomeName> = OutcomeName::ALL
        .iter()
        .map(|name| OutcomeName::of(name.name()))
        .collect();

    // Assert
    assert_eq!(read, OutcomeName::ALL);
}
