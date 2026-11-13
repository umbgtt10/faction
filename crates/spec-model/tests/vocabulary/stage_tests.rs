// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_spec_model::vocabulary::stage::Stage;

#[test]
fn name_of_each_stage_is_the_vocabulary_s() {
    // Arrange & Act
    let names: Vec<&str> = Stage::ALL.iter().map(|stage| stage.name()).collect();

    // Assert
    assert_eq!(names, ["Initial", "Pinging", "Collecting", "Bootstrapped"]);
}

#[test]
#[should_panic(expected = "a stage")]
fn of_a_name_no_stage_has_panics() {
    // Arrange & Act & Assert
    let _ = Stage::of("Concluded");
}

#[test]
fn of_each_stage_s_name_reads_that_stage() {
    // Arrange & Act
    let read: Vec<Stage> = Stage::ALL
        .iter()
        .map(|stage| Stage::of(stage.name()))
        .collect();

    // Assert
    assert_eq!(read, Stage::ALL);
}
