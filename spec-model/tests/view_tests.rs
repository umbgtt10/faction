// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::node_state::NodeState;
use faction_spec_model::view::View;
use faction_spec_model::vocabulary::stage::Stage;
use serde_json::json;

#[test]
fn of_a_bootstrapped_node_past_its_deadline_is_bootstrapped_and_says_so() {
    // Arrange
    let state = NodeState {
        deadline_missed: true,
        ..NodeStates::of(Stage::Bootstrapped, &["2"], &["1", "2"])
    };

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(
        view,
        json!({
            "peer_state": {"Bootstrapped": {}},
            "is_pinging_completed": true,
            "pinging_peers": ["2"],
            "collecting_peers": ["1", "2"],
            "required_count": "2",
            "deadline_missed": true,
            "members": ["1", "2", "3"],
        })
    );
}

#[test]
fn of_a_collecting_node_shows_its_stage_and_its_pinging_completed() {
    // Arrange
    let state = NodeStates::of(Stage::Collecting, &["3"], &["1"]);

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(view["peer_state"], json!({"Collecting": {}}));
    assert_eq!(view["is_pinging_completed"], json!(true));
}

#[test]
fn of_a_fresh_node_is_fresh() {
    // Arrange
    let state = NodeStates::of(Stage::Initial, &[], &[]);

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(
        view,
        json!({
            "peer_state": {"Fresh": {}},
            "is_pinging_completed": false,
            "pinging_peers": [],
            "collecting_peers": [],
            "required_count": "2",
            "deadline_missed": false,
            "members": ["1", "2", "3"],
        })
    );
}

#[test]
fn of_a_pinging_node_past_its_deadline_is_timed_out() {
    // Arrange
    let state = NodeState {
        deadline_missed: true,
        ..NodeStates::of(Stage::Pinging, &["2"], &[])
    };

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(view["peer_state"], json!({"TimedOut": {}}));
    assert_eq!(view["is_pinging_completed"], json!(false));
    assert_eq!(view["deadline_missed"], json!(true));
}
