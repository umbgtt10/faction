// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::addition::Addition;
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
            "addition": {"None": {}},
            "suspected": [],
            "misses": [],
            "last_probed": {"None": {}},
            "quorum": {"Held": {}},
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
fn of_a_concluded_node_holding_its_quorum_says_held() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(view["quorum"], json!({"Held": {}}));
}

#[test]
fn of_a_concluded_node_short_of_its_count_says_its_quorum_lost() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["2", "3"]),
        ..NodeStates::concluded()
    };

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(view["quorum"], json!({"Lost": {}}));
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
            "addition": {"None": {}},
            "suspected": [],
            "misses": [],
            "last_probed": {"None": {}},
            "quorum": {"NotTracked": {}},
        })
    );
}

#[test]
fn of_a_node_lists_its_missed_periods_in_the_members_order() {
    // Arrange
    let state = NodeState {
        misses: NodeStates::misses(&[("3", 1), ("2", 2)]),
        ..NodeStates::concluded()
    };

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(
        view["misses"],
        json!([{"peer_id": "2", "misses": "2"}, {"peer_id": "3", "misses": "1"}])
    );
}

#[test]
fn of_a_node_that_probed_names_the_member_probed_last() {
    // Arrange
    let state = NodeState {
        last_probed: Some(String::from("3")),
        ..NodeStates::concluded()
    };

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(view["last_probed"], json!({"Probed": {"peer_id": "3"}}));
}

#[test]
fn of_a_node_that_suspects_lists_its_suspects() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["3", "2"]),
        ..NodeStates::concluded()
    };

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(view["suspected"], json!(["3", "2"]));
}

#[test]
fn of_a_node_whose_addition_is_acknowledged_names_its_peer() {
    // Arrange
    let state = NodeStates::reconfiguring(Addition::Acknowledged(String::from("4")));

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(view["addition"], json!({"Acknowledged": {"peer_id": "4"}}));
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

#[test]
fn of_a_reconfiguring_node_is_bootstrapped_with_its_addition_requested() {
    // Arrange
    let state = NodeStates::reconfiguring(Addition::Requested(String::from("4")));

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(
        view,
        json!({
            "peer_state": {"Bootstrapped": {}},
            "is_pinging_completed": true,
            "pinging_peers": [],
            "collecting_peers": ["1"],
            "required_count": "2",
            "deadline_missed": false,
            "members": ["1", "2", "3"],
            "addition": {"Requested": {"peer_id": "4"}},
            "suspected": [],
            "misses": [],
            "last_probed": {"None": {}},
            "quorum": {"Held": {}},
        })
    );
}

#[test]
fn of_a_reconfiguring_node_past_its_deadline_is_bootstrapped_and_says_so() {
    // Arrange
    let state = NodeState {
        deadline_missed: true,
        ..NodeStates::reconfiguring(Addition::Requested(String::from("4")))
    };

    // Act
    let view = View::of(&state);

    // Assert
    assert_eq!(view["peer_state"], json!({"Bootstrapped": {}}));
    assert_eq!(view["deadline_missed"], json!(true));
}
