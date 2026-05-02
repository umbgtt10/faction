// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
