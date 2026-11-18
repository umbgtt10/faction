// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::node_state::NodeState;
use faction_spec_model::reconfiguration::Reconfiguration;
use faction_spec_model::table::next::Next;
use faction_spec_model::vocabulary::stage::Stage;

// Concluded with `changes` in flight, bound to two.
fn bound_to_two(changes: &[&str]) -> NodeState {
    NodeState {
        reconfigurations: changes
            .iter()
            .map(|peer| Reconfiguration::AdditionRequested(String::from(*peer)))
            .collect(),
        changes_in_flight: 2,
        ..NodeStates::of(Stage::Reconfiguring, &[], &["1"])
    }
}

#[test]
fn of_each_name_reads_its_next_stage() {
    // Arrange & Act
    let read = [
        Next::of("unchanged"),
        Next::of("Coordinating"),
        Next::of("changes-in-flight"),
    ];

    // Assert
    assert_eq!(
        read,
        [
            Next::Unchanged,
            Next::Stage(Stage::Coordinating),
            Next::ChangesInFlight
        ]
    );
}

#[test]
fn stage_by_the_changes_in_flight_is_bootstrapped_coordinating_or_reconfiguring() {
    // Arrange
    let states = [
        bound_to_two(&[]),
        bound_to_two(&["4"]),
        bound_to_two(&["4", "5"]),
    ];

    // Act
    let stages = states.map(|state| Next::ChangesInFlight.stage(&state));

    // Assert
    assert_eq!(
        stages,
        [
            Stage::Bootstrapped,
            Stage::Coordinating,
            Stage::Reconfiguring
        ]
    );
}

#[test]
fn stage_named_is_that_stage_and_unchanged_the_changed_state_s() {
    // Arrange
    let state = bound_to_two(&["4"]);

    // Act
    let stages = [
        Next::Stage(Stage::Removed).stage(&state),
        Next::Unchanged.stage(&state),
    ];

    // Assert
    assert_eq!(stages, [Stage::Removed, Stage::Reconfiguring]);
}
