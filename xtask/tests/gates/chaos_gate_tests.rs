// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_command_runner::FakeCommandRunner;
use std::env::temp_dir;
use std::fs::remove_dir_all;
use std::path::PathBuf;
use std::process::id;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use xtask::artifacts::run_log_directory::RunLogDirectory;
use xtask::gates::chaos_gate::ChaosGate;
use xtask::gates::gate::Gate;
use xtask::gates::gate_failure::GateFailure;
use xtask::process::launch_failure::LaunchFailure;

const RUN: &str = "2026-11-16_19-49-16";

fn gate<'a>(runner: &'a FakeCommandRunner, logs: &'a RunLogDirectory) -> ChaosGate<'a> {
    ChaosGate::new(
        runner,
        logs,
        String::from(RUN),
        String::from("faction-chaos"),
        vec![String::from("replay"), String::from("81")],
    )
}

// Under the platform's own temp directory, numbered within the process, whose
// id it carries, so two tests never share one.
fn log_root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let number = NEXT.fetch_add(1, Ordering::Relaxed);
    temp_dir().join(format!("chaos-gate-tests-{}-{number}", id()))
}

#[test]
fn label_names_the_chaos_command_it_runs() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let logs = RunLogDirectory::new(log_root(), 50);

    // Act
    let label = gate(&runner, &logs).label();

    // Assert
    assert_eq!(label, "faction chaos replay 81");
}

#[test]
fn run_of_a_chaos_run_that_fails_names_its_exit_code() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_streaming_code(Some(1))
        .with_capturing_codes(&[Some(0)]);
    let root = log_root();
    let logs = RunLogDirectory::new(root.clone(), 50);

    // Act
    let ran = gate(&runner, &logs).run();

    // Assert
    remove_dir_all(&root).ok();
    assert_eq!(ran, Err(GateFailure::ChaosFailed { exit_code: Some(1) }));
}

#[test]
fn run_prepares_the_run_directory_builds_the_node_binary_then_runs_the_chaos_binary() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let root = log_root();
    let logs = RunLogDirectory::new(root.clone(), 50);

    // Act
    let ran = gate(&runner, &logs).run();

    // Assert
    let prepared = root.join(RUN).is_dir();
    remove_dir_all(&root).ok();
    assert_eq!(ran, Ok(()));
    assert!(prepared);
    assert_eq!(
        runner.programs(),
        vec![String::from("cargo"), String::from("cargo")]
    );
    assert_eq!(
        runner.calls(),
        vec![
            [
                "build",
                "-q",
                "--release",
                "-p",
                "faction-system-tests",
                "--bin",
                "faction-node"
            ]
            .map(String::from)
            .to_vec(),
            [
                "run",
                "-q",
                "--release",
                "-p",
                "faction-chaos",
                "--",
                "replay",
                "81"
            ]
            .map(String::from)
            .to_vec(),
        ]
    );
}

#[test]
fn run_when_cargo_will_not_launch_fails_with_the_launch_failure() {
    // Arrange
    let runner = FakeCommandRunner::new().with_launch_failure();
    let root = log_root();
    let logs = RunLogDirectory::new(root.clone(), 50);

    // Act
    let ran = gate(&runner, &logs).run();

    // Assert
    remove_dir_all(&root).ok();
    assert_eq!(
        ran,
        Err(GateFailure::Launch(LaunchFailure::new(
            "cargo",
            "refused by the fake"
        )))
    );
}

#[test]
fn run_when_the_node_binary_will_not_build_fails_without_running_the_chaos() {
    // Arrange
    let runner = FakeCommandRunner::new().with_capturing_code(Some(101));
    let root = log_root();
    let logs = RunLogDirectory::new(root.clone(), 50);

    // Act
    let ran = gate(&runner, &logs).run();

    // Assert
    remove_dir_all(&root).ok();
    assert_eq!(
        ran,
        Err(GateFailure::CouldNotRun {
            exit_code: Some(101)
        })
    );
    assert_eq!(runner.calls().len(), 1);
}
