// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::harness::coordinators::Coordinators;
use faction_chaos::model::change::Change;
use faction_chaos::model::planned_change::PlannedChange;
use faction_chaos::model::scenario::Scenario;

fn planned(change: Change) -> PlannedChange {
    PlannedChange {
        change,
        due: 0,
        after: None,
    }
}

#[test]
fn of_a_scenario_names_its_genesis_members_and_newcomers_no_change_removes() {
    // Arrange
    let scenario = Scenario {
        seed: 0,
        nodes: 3,
        required_count: 1,
        suspicion_threshold: 1,
        changes: vec![
            planned(Change::Addition { peer_id: 3 }),
            planned(Change::Removal { peer_id: 1 }),
            planned(Change::Addition { peer_id: 1 }),
            planned(Change::RequiredCount { required_count: 2 }),
            planned(Change::Addition { peer_id: 4 }),
        ],
        faults: Vec::new(),
        restarts: Vec::new(),
        delivery_lag: 0,
        duplicates: 0,
        chaos_rounds: 10,
        quiescence_rounds: 10,
    };

    // Act
    let coordinators = Coordinators::of(&scenario);

    // Assert
    assert_eq!(coordinators, vec![0, 2, 3, 4]);
}
