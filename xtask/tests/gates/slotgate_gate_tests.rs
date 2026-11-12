// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_command_runner::FakeCommandRunner;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::remove_file;
use std::fs::write;
use std::path::Path;
use std::path::PathBuf;
use std::process::id;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use xtask::artifacts::run_log_directory::RunLogDirectory;
use xtask::gates::gate::Gate;
use xtask::gates::gate_failure::GateFailure;
use xtask::gates::slotgate_gate::SlotgateGate;
use xtask::process::command_outcome::CommandOutcome;
use xtask::process::command_runner::CommandRunner;
use xtask::process::launch_failure::LaunchFailure;

const RUN: &str = "2026-11-05_10-20-24";

// Lists the suite into a run directory that already holds a directory where
// the job list goes, so writing the list fails the way a full disk would.
struct BlockedJobsRunner {
    jobs_file: PathBuf,
}

impl CommandRunner for BlockedJobsRunner {
    fn is_available(&self, _binary_name: &str) -> bool {
        true
    }

    fn run_streaming(
        &self,
        _program: &str,
        _args: &[String],
    ) -> Result<Option<i32>, LaunchFailure> {
        unreachable!("a gate with no job list runs no suite")
    }

    fn run_capturing(
        &self,
        _program: &str,
        _args: &[String],
    ) -> Result<CommandOutcome, LaunchFailure> {
        create_dir_all(&self.jobs_file).expect("failed to block the job list");
        Ok(CommandOutcome::new(Some(0), String::new(), String::new()))
    }
}

fn gate<'a>(runner: &'a FakeCommandRunner, logs: &'a RunLogDirectory) -> SlotgateGate<'a> {
    gate_with(runner, logs)
}

fn gate_with<'a>(runner: &'a dyn CommandRunner, logs: &'a RunLogDirectory) -> SlotgateGate<'a> {
    SlotgateGate::new(
        runner,
        logs,
        String::from(RUN),
        String::from("System-level integration tests (max-parallel 8)"),
        String::from("faction-system-tests"),
        String::from("all_tests"),
        String::from("8"),
        String::from("120"),
    )
}

fn log_directory(root: &Path) -> RunLogDirectory {
    RunLogDirectory::new(root.to_path_buf(), 30)
}

// Under the platform's own temp directory so the suite runs the same on all
// three runners, and numbered within the process, whose id it carries, so two
// tests never share one.
fn log_root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let number = NEXT.fetch_add(1, Ordering::Relaxed);
    temp_dir().join(format!("slotgate-gate-tests-{}-{number}", id()))
}

// Writes a job log the way slotgate does, at the moment it is invoked. That is
// what makes the ordering observable: prepare refuses a run directory that is
// already there, so a log written during the run only survives to the
// assertion, and the run only passes, if the directory was prepared before the
// run rather than after it.
struct LogWritingRunner {
    log_path: PathBuf,
}

impl LogWritingRunner {
    fn new(log_path: &Path) -> Self {
        Self {
            log_path: log_path.to_path_buf(),
        }
    }
}

impl CommandRunner for LogWritingRunner {
    fn is_available(&self, _binary_name: &str) -> bool {
        true
    }

    fn run_streaming(
        &self,
        _program: &str,
        _args: &[String],
    ) -> Result<Option<i32>, LaunchFailure> {
        create_dir_all(&self.log_path).expect("slotgate makes its own log directory");
        write(self.log_path.join("cluster_election_tests.log"), "this run")
            .expect("failed to write a job log");
        Ok(Some(0))
    }

    fn run_capturing(
        &self,
        _program: &str,
        _args: &[String],
    ) -> Result<CommandOutcome, LaunchFailure> {
        Ok(CommandOutcome::new(
            Some(0),
            String::from("cluster_election_tests::elects: test\n"),
            String::new(),
        ))
    }
}

// The argument list slotgate is handed, read back as a map so an assertion can
// name the flag it cares about instead of an index that shifts whenever the
// list gains a flag.
fn value_after(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

#[test]
fn label_names_the_system_test_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let logs = log_directory(&log_root());

    // Act
    let label = gate(&runner, &logs).label();

    // Assert
    assert_eq!(label, "System-level integration tests (max-parallel 8)");
}

// The pre-build compiles the suite once; the run then executes one job per
// test against that build. Both must name the same package and target, or the
// run rebuilds what the pre-build already made -- once per job, in parallel.
#[test]
fn run_builds_and_executes_the_same_test_target() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let logs = log_directory(&log_root());

    // Act
    let _ = gate(&runner, &logs).run();

    // Assert
    assert_eq!(
        runner.calls()[0],
        vec![
            "test",
            "-p",
            "faction-system-tests",
            "--test",
            "all_tests",
            "--",
            "--list"
        ]
    );
    let args = &runner.calls()[1];
    assert_eq!(
        value_after(args, "--pre-build-args").as_deref(),
        Some("test,-p,faction-system-tests,--test,all_tests,--no-run,--message-format=json")
    );
    assert_eq!(
        value_after(args, "--program-args").as_deref(),
        Some("test,-p,faction-system-tests,--test,all_tests,{job},--,--test-threads=1,--exact")
    );
    assert_eq!(
        value_after(args, "--pre-build-target-name").as_deref(),
        Some("all_tests")
    );
    assert_eq!(
        value_after(args, "--pre-build-program").as_deref(),
        Some("cargo")
    );
    assert_eq!(value_after(args, "--program").as_deref(), Some("cargo"));
}

// The literal token slotgate substitutes per job. Spelled out because a
// formatting pass that treated it as a placeholder would leave every job
// running the whole suite, and every job would still pass.
#[test]
fn run_leaves_the_job_token_for_slotgate_to_substitute() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let logs = log_directory(&log_root());

    // Act
    let _ = gate(&runner, &logs).run();

    // Assert
    let program_args =
        value_after(&runner.calls()[1], "--program-args").expect("--program-args is passed");
    assert!(program_args.contains("{job}"));
}

// The binary's own list names the cases rstest expands, which a scan of the
// source for `#[test]` misses: one job per test, in a file slotgate reads.
#[test]
fn run_lists_the_test_binary_jobs_into_the_jobs_file() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(
        "convergence_tests::converges::case_1_task: test\njoining_tests::joins: test\nbench_one: benchmark\n",
    );
    let root = log_root();
    let logs = log_directory(&root);

    // Act
    let _ = gate(&runner, &logs).run();

    // Assert
    let jobs = read_to_string(root.join(RUN).join("jobs.txt")).expect("the job list was written");
    assert_eq!(
        jobs,
        "convergence_tests::converges::case_1_task\njoining_tests::joins\n"
    );
}

#[test]
fn run_lists_with_cargo_and_runs_with_slotgate() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let logs = log_directory(&log_root());

    // Act
    let _ = gate(&runner, &logs).run();

    // Assert -- the four house gates are cargo subcommands and this one is not:
    // cargo only lists the suite, and a copy of a house gate would reach for
    // cargo to run it as well and still pass every other assertion here.
    assert_eq!(
        runner.programs(),
        vec![String::from("cargo"), String::from("slotgate")]
    );
}

#[test]
fn run_passes_the_bounded_parallelism_settings_to_the_tool() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let root = log_root();
    let logs = log_directory(&root);

    // Act
    let _ = gate(&runner, &logs).run();

    // Assert
    let args = &runner.calls()[1];
    assert_eq!(value_after(args, "--max-parallel").as_deref(), Some("8"));
    assert_eq!(value_after(args, "--timeout-secs").as_deref(), Some("120"));
    assert_eq!(
        value_after(args, "--jobs-file").map(PathBuf::from),
        Some(root.join(RUN).join("jobs.txt"))
    );
    assert_eq!(
        value_after(args, "--log-dir").map(PathBuf::from),
        Some(root.join(RUN))
    );
}

// slotgate writes one log per job. A log left by a job that no longer exists
// would read afterwards as a result of this run, so the run writes to a
// directory of its own -- and the last run keeps what it wrote, because with
// --random a failed order may not come round again, and its job logs are the
// only record of what the cluster did.
#[test]
fn run_prepares_a_run_directory_of_its_own_beside_the_last_runs() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let root = log_root();
    let previous = root.join("2026-11-04_09-00-00");
    create_dir_all(&previous).expect("failed to create the previous run");
    write(previous.join("stale.log"), "a previous run").expect("failed to write stale log");
    let logs = log_directory(&root);

    // Act
    let _ = gate(&runner, &logs).run();

    // Assert -- the run's directory holds this run's job list and nothing older
    let written: Vec<String> = read_dir(root.join(RUN))
        .expect("the run's directory was made")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(written, vec![String::from("jobs.txt")]);
    assert!(previous.join("stale.log").exists());
}

// The name of the test above says only what it proves. This is the ordering
// itself: slotgate writes one log per job *into* the run's directory, so a
// directory prepared after the suite instead of before would be one slotgate
// had already made, and prepare refuses a directory that is already there.
#[test]
fn run_prepares_the_run_directory_before_the_suite_rather_than_after() {
    // Arrange
    let root = log_root();
    let logs = log_directory(&root);
    let runner = LogWritingRunner::new(&root.join(RUN));
    let gate = SlotgateGate::new(
        &runner,
        &logs,
        String::from(RUN),
        String::from("System-level integration tests (max-parallel 8)"),
        String::from("faction-system-tests"),
        String::from("all_tests"),
        String::from("8"),
        String::from("120"),
    );

    // Act
    let result = gate.run();

    // Assert -- this run's log survived, in the directory made for it
    assert_eq!(result, Ok(()));
    assert!(root.join(RUN).join("cluster_election_tests.log").exists());
}

// Order dependence between tests is a real defect, and a fixed order hides it
// until somebody reorders something. These tests spawn processes, bind ports
// and write journals, so there is plenty for one to leave behind for the next.
#[test]
fn run_shuffles_the_job_order() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let logs = log_directory(&log_root());

    // Act
    let _ = gate(&runner, &logs).run();

    // Assert
    assert!(runner.calls()[1].contains(&String::from("--random")));
}

#[test]
fn run_when_slotgate_will_not_launch_fails_with_the_launch_failure() {
    // Arrange
    let runner = FakeCommandRunner::new().with_launch_failure_of("slotgate");
    let logs = log_directory(&log_root());

    // Act
    let result = gate(&runner, &logs).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::Launch(LaunchFailure::new(
            "slotgate",
            "refused by the fake"
        )))
    );
}

#[test]
fn run_when_the_jobs_file_cannot_be_written_fails_without_running_the_suite() {
    // Arrange
    let root = log_root();
    let logs = log_directory(&root);
    let jobs_file = root.join(RUN).join("jobs.txt");
    let runner = BlockedJobsRunner {
        jobs_file: jobs_file.clone(),
    };

    // Act
    let result = gate_with(&runner, &logs).run();

    // Assert
    assert!(
        matches!(&result, Err(GateFailure::JobsFile { path, .. }) if *path == jobs_file.to_string_lossy()),
        "{result:?}"
    );
}

// A file where the log root should be: the gate stops there, before a suite
// whose logs it would have nowhere of their own to put.
#[test]
fn run_when_the_run_directory_cannot_be_prepared_fails_without_running_the_suite() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let root = log_root();
    write(&root, "not a directory").expect("failed to write scratch file");
    let logs = log_directory(&root);

    // Act
    let result = gate(&runner, &logs).run();

    // Assert
    assert!(
        matches!(result, Err(GateFailure::LogDirectory(_))),
        "{result:?}"
    );
    assert!(runner.calls().is_empty());
    remove_file(&root).ok();
}

#[test]
fn run_with_a_failing_listing_fails_without_running_the_suite() {
    // Arrange
    let runner = FakeCommandRunner::new().with_capturing_code(Some(101));
    let logs = log_directory(&log_root());

    // Act
    let result = gate(&runner, &logs).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::CouldNotRun {
            exit_code: Some(101)
        })
    );
    assert_eq!(runner.programs(), vec![String::from("cargo")]);
}

#[test]
fn run_with_a_non_zero_exit_code_reports_a_failing_system_test() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_streaming_code(Some(1))
        .with_capturing_code(Some(0));
    let logs = log_directory(&log_root());

    // Act
    let result = gate(&runner, &logs).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::SystemTestFailed { exit_code: Some(1) })
    );
}

#[test]
fn run_with_a_zero_exit_code_returns_ok() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let logs = log_directory(&log_root());

    // Act
    let result = gate(&runner, &logs).run();

    // Assert
    assert_eq!(result, Ok(()));
}

// None means slotgate was killed by a signal rather than exiting, which is a
// failure and not an unknown.
#[test]
fn run_with_no_exit_code_reports_a_failing_system_test() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_streaming_code(None)
        .with_capturing_code(Some(0));
    let logs = log_directory(&log_root());

    // Act
    let result = gate(&runner, &logs).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::SystemTestFailed { exit_code: None })
    );
}

#[test]
fn run_with_slotgate_missing_reports_how_to_install_it() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);
    let logs = log_directory(&log_root());

    // Act
    let result = gate(&runner, &logs).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::NotInstalled {
            binary: String::from("slotgate")
        })
    );
    assert!(runner.calls().is_empty());
}
