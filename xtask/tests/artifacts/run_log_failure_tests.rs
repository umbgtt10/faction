// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use xtask::artifacts::run_log_failure::RunLogFailure;

#[test]
fn to_string_of_a_create_failure_names_the_path_and_carries_the_reason() {
    // Arrange
    let failure = RunLogFailure::Create {
        path: PathBuf::from("logs/2026-11-05_10-20-24"),
        reason: String::from("File exists (os error 17)"),
    };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(
        shown,
        "failed to create logs/2026-11-05_10-20-24: File exists (os error 17)"
    );
}

#[test]
fn to_string_of_a_read_failure_names_the_path_and_carries_the_reason() {
    // Arrange
    let failure = RunLogFailure::Read {
        path: PathBuf::from("logs"),
        reason: String::from("permission denied"),
    };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(shown, "failed to read logs: permission denied");
}

#[test]
fn to_string_of_a_remove_failure_names_the_path_and_carries_the_reason() {
    // Arrange
    let failure = RunLogFailure::Remove {
        path: PathBuf::from("logs/2026-08-20_10-00-00"),
        reason: String::from("permission denied"),
    };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(
        shown,
        "failed to remove logs/2026-08-20_10-00-00: permission denied"
    );
}
