// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::fs::write;
use std::path::Path;

use crate::artifacts::run_log_directory::RunLogDirectory;
use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;

// Not a cargo subcommand, unlike the four house gates: slotgate is a standalone
// binary and is invoked as one.
const BINARY: &str = "slotgate";

const JOBS_FILE: &str = "jobs.txt";

pub struct SlotgateGate<'a> {
    runner: &'a dyn CommandRunner,
    log_directory: &'a RunLogDirectory,
    run_name: String,
    label: String,
    package: String,
    test_target: String,
    max_parallel: String,
    timeout_secs: String,
}

impl<'a> SlotgateGate<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        runner: &'a dyn CommandRunner,
        log_directory: &'a RunLogDirectory,
        run_name: String,
        label: String,
        package: String,
        test_target: String,
        max_parallel: String,
        timeout_secs: String,
    ) -> Self {
        Self {
            runner,
            log_directory,
            run_name,
            label,
            package,
            test_target,
            max_parallel,
            timeout_secs,
        }
    }

    // Built once and read by both the pre-build and the run, so the two cannot
    // drift onto different packages or targets.
    fn cargo_test_args(&self, trailing: &[&str]) -> Vec<String> {
        let mut args = vec![
            String::from("test"),
            String::from("-p"),
            self.package.clone(),
            String::from("--test"),
            self.test_target.clone(),
        ];
        args.extend(trailing.iter().map(|part| String::from(*part)));
        args
    }

    fn write_jobs(&self, jobs_file: &Path) -> Result<(), GateFailure> {
        let listing = self
            .runner
            .run_capturing("cargo", &self.cargo_test_args(&["--", "--list"]))?;
        if !listing.is_success() {
            return Err(GateFailure::CouldNotRun {
                exit_code: listing.exit_code(),
            });
        }
        let jobs: String = listing
            .stdout()
            .lines()
            .filter_map(|line| line.strip_suffix(": test"))
            .map(|name| format!("{name}\n"))
            .collect();
        write(jobs_file, jobs).map_err(|error| GateFailure::JobsFile {
            path: jobs_file.to_string_lossy().into_owned(),
            reason: error.to_string(),
        })
    }
}

impl Gate for SlotgateGate<'_> {
    fn label(&self) -> String {
        self.label.clone()
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.runner.is_available(BINARY) {
            return Err(GateFailure::NotInstalled {
                binary: String::from(BINARY),
            });
        }

        let run_directory = self.log_directory.prepare(&self.run_name)?;

        // The job list is the test binary's own: `--list` names every test the
        // harness will run, the cases rstest expands among them, which a scan
        // of the source for `#[test]` does not see. The names go to slotgate in
        // a file rather than on its command line, which stops spawning once a
        // suite grows past what a platform takes.
        let jobs_file = run_directory.join(JOBS_FILE);
        self.write_jobs(&jobs_file)?;

        // --random shuffles every run, from a seed slotgate draws and prints.
        // Order dependence between tests is a real defect and a fixed order
        // hides it until somebody reorders something; these suites spawn
        // processes, bind ports and write journals, so there is plenty for one
        // test to leave behind for the next. The cost is that a red run no
        // longer always means the change -- a new order can surface a latent
        // bug on an unrelated change. That is the point of the feature rather
        // than a flaw in it, and the printed seed replays the exact order.
        let args = vec![
            String::from("--random"),
            String::from("--max-parallel"),
            self.max_parallel.clone(),
            String::from("--timeout-secs"),
            self.timeout_secs.clone(),
            String::from("--log-dir"),
            run_directory.to_string_lossy().into_owned(),
            String::from("--pre-build-program"),
            String::from("cargo"),
            String::from("--pre-build-args"),
            self.cargo_test_args(&["--no-run", "--message-format=json"])
                .join(","),
            String::from("--pre-build-target-name"),
            self.test_target.clone(),
            String::from("--program"),
            String::from("cargo"),
            String::from("--program-args"),
            self.cargo_test_args(&["{job}", "--", "--test-threads=1", "--exact"])
                .join(","),
            String::from("--jobs-file"),
            jobs_file.to_string_lossy().into_owned(),
        ];

        match self.runner.run_streaming(BINARY, &args)? {
            Some(0) => Ok(()),
            exit_code => Err(GateFailure::SystemTestFailed { exit_code }),
        }
    }
}
