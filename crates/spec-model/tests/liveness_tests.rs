// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::liveness::Liveness;
use faction_spec_model::node_state::NodeState;

fn of_five() -> NodeState {
    NodeState {
        members: NodeStates::peers(&["1", "2", "3", "4", "5"]),
        ..NodeStates::concluded()
    }
}

#[test]
fn helpers_leaves_out_the_suspected_members() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["4"]),
        ..of_five()
    };

    // Act
    let helpers = Liveness::helpers(&state, "3");

    // Assert
    assert_eq!(helpers, NodeStates::peers(&["5", "2"]));
}

#[test]
fn helpers_lists_the_members_after_the_missed_one_past_the_last_back_to_the_first() {
    // Arrange
    let state = of_five();

    // Act
    let helpers = Liveness::helpers(&state, "3");

    // Assert
    assert_eq!(helpers, NodeStates::peers(&["4", "5", "2"]));
}

#[test]
fn holds_quorum_once_the_members_not_suspected_fall_short_of_the_count_is_false() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["2", "3"]),
        ..NodeStates::concluded()
    };

    // Act
    let holds = Liveness::holds_quorum(&state);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_quorum_while_the_members_not_suspected_reach_the_count_is_true() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["2"]),
        ..NodeStates::concluded()
    };

    // Act
    let holds = Liveness::holds_quorum(&state);

    // Assert
    assert!(holds);
}

#[test]
fn misses_of_a_member_counted_is_its_count() {
    // Arrange
    let state = NodeState {
        misses: NodeStates::misses(&[("2", 2)]),
        ..NodeStates::concluded()
    };

    // Act
    let misses = Liveness::misses(&state, "2");

    // Assert
    assert_eq!(misses, 2);
}

#[test]
fn misses_of_a_member_that_missed_nothing_is_zero() {
    // Arrange
    let state = NodeState {
        misses: NodeStates::misses(&[("2", 2)]),
        ..NodeStates::concluded()
    };

    // Act
    let misses = Liveness::misses(&state, "3");

    // Assert
    assert_eq!(misses, 0);
}

#[test]
fn next_to_probe_after_a_member_is_the_one_after_it() {
    // Arrange
    let state = NodeState {
        last_probed: Some(String::from("2")),
        ..NodeStates::concluded()
    };

    // Act
    let next = Liveness::next_to_probe(&state);

    // Assert
    assert_eq!(next, Some(String::from("3")));
}

#[test]
fn next_to_probe_after_the_last_member_is_the_first_but_the_node() {
    // Arrange
    let state = NodeState {
        last_probed: Some(String::from("3")),
        ..NodeStates::concluded()
    };

    // Act
    let next = Liveness::next_to_probe(&state);

    // Assert
    assert_eq!(next, Some(String::from("2")));
}

#[test]
fn next_to_probe_of_a_node_alone_is_no_one() {
    // Arrange
    let state = NodeState {
        members: NodeStates::peers(&["1"]),
        ..NodeStates::concluded()
    };

    // Act
    let next = Liveness::next_to_probe(&state);

    // Assert
    assert_eq!(next, None);
}

#[test]
fn next_to_probe_with_no_one_probed_is_the_first_member_but_the_node() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let next = Liveness::next_to_probe(&state);

    // Assert
    assert_eq!(next, Some(String::from("2")));
}
