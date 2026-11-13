// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_spec_model::vocabulary::command_name::CommandName;

#[test]
fn name_of_each_command_is_the_vocabulary_s_in_the_commands_order() {
    // Arrange & Act
    let names: Vec<&str> = CommandName::ALL.iter().map(|name| name.name()).collect();

    // Assert
    assert_eq!(
        names,
        [
            "ParticipationObserved",
            "ReadyObserved",
            "LocalParticipationCompleted",
            "DeadlineExpired",
            "JoinRequested",
            "JoinApproved",
            "JoinRejected",
            "Probe",
        ]
    );
}

#[test]
#[should_panic(expected = "a command")]
fn of_a_name_no_command_has_panics() {
    // Arrange & Act & Assert
    let _ = CommandName::of("Ping");
}

#[test]
fn of_each_command_s_name_reads_that_command() {
    // Arrange & Act
    let read: Vec<CommandName> = CommandName::ALL
        .iter()
        .map(|name| CommandName::of(name.name()))
        .collect();

    // Assert
    assert_eq!(read, CommandName::ALL);
}
