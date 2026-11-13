// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::command::Command;
use faction_spec_model::node_state::NodeState;
use faction_spec_model::reconfiguration::Reconfiguration;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::outcome_name::OutcomeName;
use faction_spec_model::vocabulary::stage::Stage;
use serde_json::json;
use serde_json::Value;

fn naming(name: CommandName, peer_id: &str) -> Command {
    Command {
        name,
        peer_id: Some(String::from(peer_id)),
        required_count: None,
    }
}

fn probe_due() -> Command {
    Command {
        name: CommandName::ProbeDue,
        peer_id: None,
        required_count: None,
    }
}

fn signal_from(peer_id: &str) -> Command {
    Command {
        name: CommandName::ReadyObserved,
        peer_id: Some(String::from(peer_id)),
        required_count: None,
    }
}

#[test]
fn encoded_of_a_liveness_outcome_names_the_command_s_peer() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let encoded =
        OutcomeName::AlreadySuspected.encoded(&naming(CommandName::ProbeMissed, "2"), &state);

    // Assert
    assert_eq!(encoded, json!({"AlreadySuspected": {"peer_id": "2"}}));
}

#[test]
fn encoded_of_a_removal_s_outcome_names_the_command_s_peer() {
    // Arrange
    let commit = naming(CommandName::RemovalCommitted, "2");
    let removal_outcomes = [
        OutcomeName::ProposeRemoval,
        OutcomeName::RemovalRefused,
        OutcomeName::RemovalWithdrawn,
        OutcomeName::MemberRemoved,
    ];

    // Act
    let encoded: Vec<Value> = removal_outcomes
        .iter()
        .map(|outcome| outcome.encoded(&commit, &NodeStates::concluded()))
        .collect();

    // Assert
    assert_eq!(
        encoded,
        [
            json!({"ProposeRemoval": {"peer_id": "2"}}),
            json!({"RemovalRefused": {"peer_id": "2"}}),
            json!({"RemovalWithdrawn": {"peer_id": "2"}}),
            json!({"MemberRemoved": {"peer_id": "2"}}),
        ]
    );
}

#[test]
fn encoded_of_an_addition_s_outcome_names_the_command_s_peer() {
    // Arrange
    let commit = Command {
        name: CommandName::AdditionCommitted,
        peer_id: Some(String::from("4")),
        required_count: None,
    };
    let addition_outcomes = [
        OutcomeName::SendMembership,
        OutcomeName::ProposeAddition,
        OutcomeName::AdditionWithdrawn,
        OutcomeName::AcknowledgementIgnored,
        OutcomeName::AbortIgnored,
    ];

    // Act
    let encoded: Vec<Value> = addition_outcomes
        .iter()
        .map(|outcome| outcome.encoded(&commit, &NodeStates::reconfiguring(Reconfiguration::None)))
        .collect();

    // Assert
    assert_eq!(
        encoded,
        [
            json!({"SendMembership": {"peer_id": "4"}}),
            json!({"ProposeAddition": {"peer_id": "4"}}),
            json!({"AdditionWithdrawn": {"peer_id": "4"}}),
            json!({"AcknowledgementIgnored": {"peer_id": "4"}}),
            json!({"AbortIgnored": {"peer_id": "4"}}),
        ]
    );
}

#[test]
fn encoded_of_an_outcome_carrying_nothing_is_a_unit() {
    // Arrange
    let completion = Command {
        name: CommandName::LocalParticipationCompleted,
        peer_id: None,
        required_count: None,
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
        required_count: None,
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
fn encoded_of_miss_counted_counts_the_period_just_missed() {
    // Arrange
    let state = NodeState {
        misses: NodeStates::misses(&[("2", 1)]),
        ..NodeStates::concluded()
    };

    // Act
    let encoded =
        OutcomeName::MissCounted.encoded(&naming(CommandName::IndirectProbesMissed, "2"), &state);

    // Assert
    assert_eq!(
        encoded,
        json!({"MissCounted": {"peer_id": "2", "misses": "2"}})
    );
}

#[test]
fn encoded_of_quorum_lost_is_a_unit() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let encoded =
        OutcomeName::QuorumLost.encoded(&naming(CommandName::IndirectProbesMissed, "2"), &state);

    // Assert
    assert_eq!(encoded, json!({"QuorumLost": {}}));
}

#[test]
fn encoded_of_required_count_set_carries_the_command_s_count() {
    // Arrange
    let new_count = Command {
        name: CommandName::RequiredCountChanged,
        peer_id: None,
        required_count: Some(3),
    };

    // Act
    let encoded = OutcomeName::RequiredCountSet
        .encoded(&new_count, &NodeStates::of(Stage::Collecting, &[], &["1"]));

    // Assert
    assert_eq!(
        encoded,
        json!({"RequiredCountSet": {"required_count": "3"}})
    );
}

#[test]
fn encoded_of_send_indirect_probes_lists_the_members_to_ask() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let encoded =
        OutcomeName::SendIndirectProbes.encoded(&naming(CommandName::ProbeMissed, "2"), &state);

    // Assert
    assert_eq!(
        encoded,
        json!({"SendIndirectProbes": {"peer_id": "2", "via": ["3"]}})
    );
}

#[test]
fn encoded_of_send_probe_names_the_member_to_probe() {
    // Arrange
    let state = NodeState {
        last_probed: Some(String::from("2")),
        ..NodeStates::concluded()
    };

    // Act
    let encoded = OutcomeName::SendProbe.encoded(&probe_due(), &state);

    // Assert
    assert_eq!(encoded, json!({"SendProbe": {"peer_id": "3"}}));
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
            "SendMembership",
            "ProposeAddition",
            "AdditionWithdrawn",
            "AcknowledgementIgnored",
            "AbortIgnored",
            "ProposeRemoval",
            "RemovalRefused",
            "RemovalWithdrawn",
            "MemberRemoved",
            "RequiredCountSet",
            "SendProbe",
            "NothingToProbe",
            "SendIndirectProbes",
            "MissCounted",
            "PeerSuspected",
            "PeerRevived",
            "AliveNoted",
            "AlreadySuspected",
            "SelfIgnored",
            "QuorumLost",
            "QuorumRestored",
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
