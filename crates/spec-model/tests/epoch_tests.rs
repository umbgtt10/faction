// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_spec_model::command::Command;
use faction_spec_model::epoch::Epoch;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::result_name::ResultName;
use serde_json::json;

fn at(name: CommandName, epoch: u64) -> Command {
    Command {
        name,
        peer_id: Some(String::from("2")),
        required_count: None,
        epoch,
        members: Vec::new(),
    }
}

#[test]
fn after_a_rejected_commit_or_a_signal_is_the_row_s_epoch() {
    // Arrange
    let rejected = at(CommandName::AdditionCommitted, 5);
    let signal = at(CommandName::ParticipationObserved, 5);

    // Act
    let after = [
        Epoch::after(&rejected, ResultName::Rejected, 3),
        Epoch::after(&signal, ResultName::Accepted, 3),
    ];

    // Assert
    assert_eq!(after, [3, 3]);
}

#[test]
fn after_an_accepted_commit_is_the_greater_of_the_two_epochs() {
    // Arrange
    let commits = [
        at(CommandName::AdditionCommitted, 5),
        at(CommandName::RemovalCommitted, 1),
        at(CommandName::RequiredCountChanged, 0),
    ];

    // Act
    let after = commits.map(|commit| Epoch::after(&commit, ResultName::Accepted, 3));

    // Assert
    assert_eq!(after, [5, 3, 3]);
}

#[test]
fn stated_of_a_payload_adds_the_epoch_but_at_0() {
    // Arrange & Act
    let stated = [
        Epoch::stated(json!({"peer_id": "2"}), 4),
        Epoch::stated(json!({"peer_id": "2"}), 0),
    ];

    // Assert
    assert_eq!(
        stated,
        [
            json!({"peer_id": "2", "epoch": "4"}),
            json!({"peer_id": "2"})
        ]
    );
}
