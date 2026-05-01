// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction_core_validation::scenario_harness::ScenarioHarness;

#[test]
fn new_creates_one_coordinator_per_peer() {
    // Arrange & Act
    let harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);

    // Assert
    assert_eq!(harness.coordinator_count(), 5);
    assert_eq!(harness.current_marker(), 0);
}

#[test]
fn advance_to_sets_current_marker() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);

    // Act
    harness.advance_to(10);

    // Assert
    assert_eq!(harness.current_marker(), 10);
}

#[test]
fn advance_by_increments_current_marker() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(3);

    // Act
    harness.advance_by(7);

    // Assert
    assert_eq!(harness.current_marker(), 10);
}
