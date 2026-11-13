// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_spec_model::vocabulary::result_name::ResultName;
use serde_json::json;

#[test]
fn encoded_of_accepted_carries_the_outcomes_the_view_and_the_admissible_commands() {
    // Arrange
    let outcomes = vec![json!({"JoinDenied": {"peer_id": "4"}})];

    // Act
    let encoded = ResultName::Accepted.encoded(outcomes, json!({"view": true}), json!(["Probe"]));

    // Assert
    assert_eq!(
        encoded,
        json!({"Accepted": {
            "outcomes": [{"JoinDenied": {"peer_id": "4"}}],
            "cluster_view": {"view": true},
            "admissible": ["Probe"],
        }})
    );
}

#[test]
fn encoded_of_probed_carries_the_view_and_the_admissible_commands_alone() {
    // Arrange & Act
    let encoded = ResultName::Probed.encoded(Vec::new(), json!({"view": true}), json!(["Probe"]));

    // Assert
    assert_eq!(
        encoded,
        json!({"Probed": {"cluster_view": {"view": true}, "admissible": ["Probe"]}})
    );
}

#[test]
fn encoded_of_rejected_carries_the_view_and_the_admissible_commands_alone() {
    // Arrange & Act
    let encoded =
        ResultName::Rejected.encoded(Vec::new(), json!({"view": false}), json!(["Probe"]));

    // Assert
    assert_eq!(
        encoded,
        json!({"Rejected": {"cluster_view": {"view": false}, "admissible": ["Probe"]}})
    );
}

#[test]
fn name_of_each_result_is_the_vocabulary_s() {
    // Arrange & Act
    let names: Vec<&str> = ResultName::ALL.iter().map(|name| name.name()).collect();

    // Assert
    assert_eq!(names, ["Accepted", "Rejected", "Probed"]);
}

#[test]
fn of_each_result_s_name_reads_that_result() {
    // Arrange & Act
    let read: Vec<ResultName> = ResultName::ALL
        .iter()
        .map(|name| ResultName::of(name.name()))
        .collect();

    // Assert
    assert_eq!(read, ResultName::ALL);
}
