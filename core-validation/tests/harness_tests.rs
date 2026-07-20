// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::vec;

use faction_core_validation::scenario_harness::ScenarioHarness;

#[test]
fn new_creates_one_coordinator_per_peer() {
    // Arrange & Act
    let harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);

    // Assert
    assert_eq!(harness.coordinator_count(), 5);
}
