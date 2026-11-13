// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use crate::support::spec_files::SpecFiles;
use faction_spec_model::command::Command;
use faction_spec_model::spec_reader::SpecReader;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::stage::Stage;

fn command(name: CommandName, peer_id: Option<&str>) -> Command {
    Command {
        name,
        peer_id: peer_id.map(String::from),
        required_count: None,
    }
}

#[test]
fn admits_of_collecting_is_every_command_but_its_own_completion_and_a_change_s_coordination() {
    // Arrange
    let table = SpecReader::table(&SpecFiles::matrix());

    // Act
    let admitted = table.admits(Stage::Collecting);

    // Assert
    assert_eq!(
        admitted,
        [
            CommandName::ParticipationObserved,
            CommandName::ReadyObserved,
            CommandName::DeadlineExpired,
            CommandName::JoinRequested,
            CommandName::JoinApproved,
            CommandName::JoinRejected,
            CommandName::AdditionCommitted,
            CommandName::RemovalCommitted,
            CommandName::RequiredCountChanged,
            CommandName::Probe,
        ]
    );
}

#[test]
fn row_of_a_command_the_stage_does_not_admit_is_not_admitted() {
    // Arrange
    let table = SpecReader::table(&SpecFiles::matrix());
    let state = NodeStates::of(Stage::Initial, &[], &[]);

    // Act
    let row = table.row(&command(CommandName::DeadlineExpired, None), &state);

    // Assert
    assert_eq!(row.id, "not-admitted");
}

#[test]
fn row_of_a_probe_is_probed() {
    // Arrange
    let table = SpecReader::table(&SpecFiles::matrix());
    let state = NodeStates::of(Stage::Bootstrapped, &[], &[]);

    // Act
    let row = table.row(&command(CommandName::Probe, None), &state);

    // Assert
    assert_eq!(row.id, "probed");
}

#[test]
fn row_of_a_repeated_participation_is_participation_duplicate() {
    // Arrange
    let table = SpecReader::table(&SpecFiles::matrix());
    let state = NodeStates::of(Stage::Pinging, &["2"], &[]);

    // Act
    let row = table.row(
        &command(CommandName::ParticipationObserved, Some("2")),
        &state,
    );

    // Assert
    assert_eq!(row.id, "participation-duplicate");
}

#[test]
fn row_of_a_stranger_s_participation_is_the_first_that_applies() {
    // Arrange
    let table = SpecReader::table(&SpecFiles::matrix());
    let state = NodeStates::of(Stage::Pinging, &[], &[]);

    // Act
    let row = table.row(
        &command(CommandName::ParticipationObserved, Some("7")),
        &state,
    );

    // Assert
    assert_eq!(row.id, "non-member-before-completion");
}

#[test]
fn row_of_the_node_s_completion_once_collecting_is_not_admitted() {
    // Arrange
    let table = SpecReader::table(&SpecFiles::matrix());
    let state = NodeStates::of(Stage::Collecting, &[], &["1"]);

    // Act
    let row = table.row(
        &command(CommandName::LocalParticipationCompleted, None),
        &state,
    );

    // Assert
    assert_eq!(row.id, "not-admitted");
}
