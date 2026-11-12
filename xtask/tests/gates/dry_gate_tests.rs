// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_command_runner::FakeCommandRunner;
use xtask::gates::dry_gate::DryGate;
use xtask::gates::gate::Gate;

fn gate(runner: &FakeCommandRunner) -> DryGate<'_> {
    DryGate::new(runner, String::from("/repo"), String::from("25"))
}

#[test]
fn label_names_the_duplication_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "Duplication");
}

// Zero ceilings and no baseline: Faction carries no duplication, so a check
// against nothing recorded fails on any.
#[test]
fn run_checks_the_path_above_the_floor_with_zero_ceilings_and_no_baseline() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    assert_eq!(
        runner.calls()[0],
        vec![
            String::from("dry4rust"),
            String::from("--path"),
            String::from("/repo"),
            String::from("--min-nodes"),
            String::from("25"),
            String::from("check"),
            String::from("--max-exact"),
            String::from("0"),
            String::from("--max-near"),
            String::from("0"),
        ]
    );
}

#[test]
fn run_with_a_zero_exit_code_returns_ok() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_ok());
}

// 1 is dry4rust's own "a ceiling was exceeded", which has to read as
// duplication found rather than as the tool failing to run.
#[test]
fn run_with_exit_code_one_reports_duplication_found() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(1));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from("duplicated code was found"))
    );
}

// 2 is a configuration dry4rust refused: the tool did not get as far as
// measuring anything.
#[test]
fn run_with_exit_code_two_reports_the_exit_code() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(2));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_err_and(|error| error.to_string().contains("exit code")));
}

#[test]
fn run_with_the_tool_missing_returns_an_install_hint() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_err_and(|error| error.to_string().contains("cargo install cargo-dry4rust")));
}
