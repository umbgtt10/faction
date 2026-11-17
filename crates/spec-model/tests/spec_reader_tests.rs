// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::command::Command;
use faction_spec_model::configuration::Configuration;
use faction_spec_model::spec_reader::SpecReader;
use faction_spec_model::table::next::Next;
use faction_spec_model::table::row::Row;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::outcome_name::OutcomeName;
use faction_spec_model::vocabulary::result_name::ResultName;
use faction_spec_model::vocabulary::stage::Stage;
use serde_json::Value;
use serde_json::json;

// A table of three rows: the probe, the node's own completion concluding, and
// an addition requested.
fn matrix() -> Value {
    json!({
        "version": "0.1.0",
        "admits": {
            "Initial": ["LocalParticipationCompleted", "Probe"],
            "Pinging": ["LocalParticipationCompleted", "Probe"],
            "Collecting": ["Probe"],
            "Bootstrapped": ["Probe"],
            "Coordinating": ["Probe"],
            "Reconfiguring": ["Probe"],
            "Removed": ["Probe"],
        },
        "rows": [
            {
                "id": "probed",
                "stages": ["Initial", "Pinging", "Collecting", "Bootstrapped"],
                "commands": ["Probe"],
                "situation": "always",
                "result": "Probed",
                "outcomes": [],
                "next": "unchanged",
            },
            {
                "id": "local-concluded",
                "stages": ["Initial", "Pinging"],
                "commands": ["LocalParticipationCompleted"],
                "situation": "with the node itself a collecting peer, the collecting peers number the required count or more",
                "result": "Accepted",
                "outcomes": ["LocalParticipationCompleted", "BroadcastLocalReady", "Concluded"],
                "next": "Bootstrapped",
            },
            {
                "id": "addition-requested",
                "stages": ["Bootstrapped"],
                "commands": ["AdditionRequested"],
                "situation": "otherwise: the addition of `peer_id` is requested",
                "result": "Accepted",
                "outcomes": ["SendMembership"],
                "next": "changes-in-flight",
            },
        ],
    })
}

#[test]
fn command_of_a_command_carrying_nothing_names_no_peer() {
    // Arrange & Act
    let command = SpecReader::command(&json!({"DeadlineExpired": {}}));

    // Assert
    assert_eq!(
        command,
        Command {
            name: CommandName::DeadlineExpired,
            peer_id: None,
            required_count: None,
            epoch: 0,
            members: Vec::new(),
        }
    );
}

#[test]
fn command_of_a_membership_reads_its_members_its_count_and_its_epoch() {
    // Arrange & Act
    let command = SpecReader::command(&json!({"MembershipReceived": {
        "members": ["2", "3", "1"],
        "required_count": "2",
        "epoch": "4",
    }}));

    // Assert
    assert_eq!(
        command,
        Command {
            name: CommandName::MembershipReceived,
            peer_id: None,
            required_count: Some(2),
            epoch: 4,
            members: NodeStates::peers(&["2", "3", "1"]),
        }
    );
}

#[test]
fn command_of_a_new_required_count_reads_the_count() {
    // Arrange & Act
    let command = SpecReader::command(&json!({"RequiredCountChanged": {"required_count": "3"}}));

    // Assert
    assert_eq!(
        command,
        Command {
            name: CommandName::RequiredCountChanged,
            peer_id: None,
            required_count: Some(3),
            epoch: 0,
            members: Vec::new(),
        }
    );
}

#[test]
fn command_of_a_peer_s_signal_names_the_command_and_its_peer() {
    // Arrange & Act
    let command =
        SpecReader::command(&json!({"ReadyObserved": {"peer_id": "18446744073709551615"}}));

    // Assert
    assert_eq!(
        command,
        Command {
            name: CommandName::ReadyObserved,
            peer_id: Some(String::from("18446744073709551615")),
            required_count: None,
            epoch: 0,
            members: Vec::new(),
        }
    );
}

#[test]
fn command_of_a_signal_reads_its_epoch() {
    // Arrange & Act
    let command =
        SpecReader::command(&json!({"ParticipationObserved": {"peer_id": "2", "epoch": "3"}}));

    // Assert
    assert_eq!(command.epoch, 3);
}

#[test]
fn command_of_a_signal_with_no_epoch_is_at_epoch_0() {
    // Arrange & Act
    let command = SpecReader::command(&json!({"ReadyObserved": {"peer_id": "2"}}));

    // Assert
    assert_eq!(command.epoch, 0);
}

#[test]
fn configuration_of_a_bound_of_changes_in_flight_reads_it() {
    // Arrange & Act
    let configuration = SpecReader::configuration(&json!({
        "instance": {"node": "1", "incarnation": "1"},
        "peers": ["1", "2"],
        "required_count": "1",
        "suspicion_threshold": "1",
        "changes_in_flight": "3",
    }));

    // Assert
    assert_eq!(configuration.changes_in_flight, 3);
}

#[test]
fn configuration_of_a_configuration_at_an_epoch_reads_it() {
    // Arrange & Act
    let configuration = SpecReader::configuration(&json!({
        "instance": {"node": "1", "incarnation": "1"},
        "peers": ["1", "2"],
        "required_count": "1",
        "suspicion_threshold": "1",
        "epoch": "7",
    }));

    // Assert
    assert_eq!(configuration.epoch, 7);
}

#[test]
fn configuration_of_a_node_keeps_its_peers_in_their_order() {
    // Arrange & Act
    let configuration = SpecReader::configuration(&json!({
        "instance": {"node": "2", "incarnation": "4"},
        "peers": ["3", "1", "2"],
        "required_count": "18446744073709551615",
        "suspicion_threshold": "3",
    }));

    // Assert
    assert_eq!(
        configuration,
        Configuration {
            peer_id: String::from("2"),
            incarnation: 4,
            peers: NodeStates::peers(&["3", "1", "2"]),
            required_count: u64::MAX,
            suspicion_threshold: 3,
            epoch: 0,
            provisional: false,
            changes_in_flight: 1,
        }
    );
}

#[test]
fn configuration_of_a_provisional_membership_reads_it() {
    // Arrange & Act
    let configuration = SpecReader::configuration(&json!({
        "instance": {"node": "5", "incarnation": "1"},
        "peers": ["1", "5"],
        "required_count": "1",
        "suspicion_threshold": "1",
        "epoch": "3",
        "provisional": true,
    }));

    // Assert
    assert!(configuration.provisional);
}

#[test]
fn table_of_a_matrix_admits_what_it_lists_for_each_stage() {
    // Arrange & Act
    let table = SpecReader::table(&matrix());

    // Assert
    assert_eq!(
        table.admits(Stage::Pinging),
        [CommandName::LocalParticipationCompleted, CommandName::Probe]
    );
    assert_eq!(table.admits(Stage::Bootstrapped), [CommandName::Probe]);
}

#[test]
fn table_of_a_matrix_reads_a_row_leaving_the_stage_unchanged_with_no_next_stage() {
    // Arrange
    let table = SpecReader::table(&matrix());
    let probe = SpecReader::command(&json!({"Probe": {}}));

    // Act
    let row = table.row(&probe, &NodeStates::of(Stage::Collecting, &[], &[]));

    // Assert
    assert_eq!(row.id, "probed");
    assert_eq!(row.next, Next::Unchanged);
    assert_eq!(row.result, ResultName::Probed);
    assert!(row.outcomes.is_empty());
}

#[test]
fn table_of_a_matrix_reads_a_row_whose_next_stage_the_changes_in_flight_make() {
    // Arrange
    let table = SpecReader::table(&matrix());
    let request = SpecReader::command(&json!({"AdditionRequested": {"peer_id": "4"}}));

    // Act
    let row = table.row(&request, &NodeStates::concluded());

    // Assert
    assert_eq!(row.id, "addition-requested");
    assert_eq!(row.next, Next::ChangesInFlight);
}

#[test]
fn table_of_a_matrix_reads_each_row_s_stages_commands_result_outcomes_and_next_stage() {
    // Arrange
    let table = SpecReader::table(&matrix());
    let completion = SpecReader::command(&json!({"LocalParticipationCompleted": {}}));

    // Act
    let row = table.row(&completion, &NodeStates::of(Stage::Pinging, &[], &["2"]));

    // Assert
    assert_eq!(
        *row,
        Row {
            id: String::from("local-concluded"),
            stages: vec![Stage::Initial, Stage::Pinging],
            commands: vec![CommandName::LocalParticipationCompleted],
            result: ResultName::Accepted,
            outcomes: vec![
                OutcomeName::LocalParticipationCompleted,
                OutcomeName::BroadcastLocalReady,
                OutcomeName::Concluded,
            ],
            next: Next::Stage(Stage::Bootstrapped),
        }
    );
}
