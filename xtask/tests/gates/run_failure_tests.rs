// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Each way faction's own gates fail, a run's end said as serge4rust says it,
// and handed to serge4rust's stage in those same words.

use serge4rust::gates::gate_failure::GateFailure;
use std::path::PathBuf;
use xtask::artifacts::run_log_failure::RunLogFailure;
use xtask::gates::run_failure::RunFailure;

#[test]
fn from_of_a_run_failure_is_a_gate_failure_in_its_own_words() {
    // Arrange
    let run = RunFailure::ChaosFailed { exit_code: Some(1) };

    // Act
    let failure = GateFailure::from(run);

    // Assert
    assert_eq!(
        failure,
        GateFailure::Own {
            reason: String::from("a chaos run failed, exit code 1")
        }
    );
}

#[test]
fn from_of_a_run_log_failure_holds_it() {
    // Arrange
    let run_log = RunLogFailure::Create {
        path: PathBuf::from("logs"),
        reason: String::from("busy"),
    };

    // Act
    let failure = RunFailure::from(run_log.clone());

    // Assert
    assert_eq!(failure, RunFailure::LogDirectory(run_log));
}

#[test]
fn to_string_of_a_failed_chaos_run_names_its_exit_code() {
    // Arrange
    let failure = RunFailure::ChaosFailed { exit_code: Some(1) };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(shown, "a chaos run failed, exit code 1");
}

#[test]
fn to_string_of_a_failed_system_test_ended_by_a_signal_says_so() {
    // Arrange
    let failure = RunFailure::SystemTestFailed { exit_code: None };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(shown, "a system test failed, terminated by signal");
}

#[test]
fn to_string_of_a_job_list_not_written_names_the_path_and_the_reason() {
    // Arrange
    let failure = RunFailure::JobsFile {
        path: String::from("logs/run/jobs.txt"),
        reason: String::from("Is a directory"),
    };

    // Act
    let message = failure.to_string();

    // Assert
    assert_eq!(
        message,
        "could not write the job list to logs/run/jobs.txt: Is a directory"
    );
}

#[test]
fn to_string_of_a_log_directory_failure_is_the_run_log_failure() {
    // Arrange
    let failure = RunFailure::LogDirectory(RunLogFailure::Create {
        path: PathBuf::from("logs"),
        reason: String::from("busy"),
    });

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(shown, "failed to create logs: busy");
}
