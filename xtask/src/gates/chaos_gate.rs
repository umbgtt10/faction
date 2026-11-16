// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::artifacts::run_log_directory::RunLogDirectory;
use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;

// The binary a process node runs, which the chaos binary's process cases
// launch: built in release, beside it, as `cargo run` builds no other
// package's binaries.
const NODE_BINARY: &str = "faction-node";
const NODE_PACKAGE: &str = "faction-system-tests";

// The chaos binary run once -- a seed replayed, a soak, or the real engine's
// list -- in a run directory of its own under logs/chaos/, prepared before it
// starts, the oldest beyond the limit removed, the process node's binary built
// beside it first: the runner hands it the run's name, as the system tests are
// handed theirs, and it writes every log of the run there.
pub struct ChaosGate<'a> {
    runner: &'a dyn CommandRunner,
    log_directory: &'a RunLogDirectory,
    run_name: String,
    package: String,
    args: Vec<String>,
}

impl<'a> ChaosGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        log_directory: &'a RunLogDirectory,
        run_name: String,
        package: String,
        args: Vec<String>,
    ) -> Self {
        Self {
            runner,
            log_directory,
            run_name,
            package,
            args,
        }
    }
}

impl Gate for ChaosGate<'_> {
    fn label(&self) -> String {
        format!("faction chaos {}", self.args.join(" "))
    }

    fn run(&self) -> Result<(), GateFailure> {
        self.log_directory.prepare(&self.run_name)?;
        let build = [
            "build",
            "-q",
            "--release",
            "-p",
            NODE_PACKAGE,
            "--bin",
            NODE_BINARY,
        ]
        .map(String::from);
        let built = self.runner.run_capturing("cargo", &build)?;
        if !built.is_success() {
            return Err(GateFailure::CouldNotRun {
                exit_code: built.exit_code(),
            });
        }
        let args: Vec<String> = ["run", "-q", "--release", "-p", &self.package, "--"]
            .into_iter()
            .map(String::from)
            .chain(self.args.iter().cloned())
            .collect();
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            exit_code => Err(GateFailure::ChaosFailed { exit_code }),
        }
    }
}
