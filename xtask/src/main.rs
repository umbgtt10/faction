// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::env::args;
use std::env::temp_dir;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::Path;
use std::path::PathBuf;
use std::process::id;
use std::process::ExitCode;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use xtask::artifacts::run_log_directory::RunLogDirectory;
use xtask::artifacts::run_stamp::RunStamp;
use xtask::gates::coverage_gate::CoverageGate;
use xtask::gates::crap_gate::CrapGate;
use xtask::gates::deny_gate::DenyGate;
use xtask::gates::dry_gate::DryGate;
use xtask::gates::gate::Gate;
use xtask::gates::gate_sequence::GateSequence;
use xtask::gates::iceberg_gate::IcebergGate;
use xtask::gates::slotgate_gate::SlotgateGate;
use xtask::gates::stern_gate::SternGate;
use xtask::gates::twin_gate::TwinGate;
use xtask::process::system_command_runner::SystemCommandRunner;
use xtask::reports::crap_report_parser::CrapReportParser;
use xtask::reports::scores::library_scorer::LibraryScorer;
use xtask::reports::scores::message_scoring::MessageScoring;

const CORE_PACKAGE: &str = "faction";
const CORE_VALIDATION_PACKAGE: &str = "faction-core-validation";
const CONFORMANCE_TIER: &str = "conformance";
const LADDER_PACKAGE: &str = "faction-ladder";
const PROTOCOL_PACKAGE: &str = "faction-protocol";
const PROTOCOL_VALIDATION_PACKAGE: &str = "faction-protocol-validation";
const SPEC_MODEL_PACKAGE: &str = "faction-spec-model";
const SYSTEM_TESTS_PACKAGE: &str = "faction-system-tests";
const XTASK_PACKAGE: &str = "xtask";
const ALL_TESTS: &str = "all_tests";
const CRAP_THRESHOLD: &str = "15";
const ICEBERG_THRESHOLD: &str = "5";
const DRY_MIN_NODES: &str = "25";
const MAX_PARALLEL: &str = "6";
const TIMEOUT_SECS: &str = "300";
const RUNS_TO_KEEP: usize = 50;

// Reading the real process argv, the clock and wiring the concrete runner are
// the things no test can reach, so they are all this binary does.
fn main() -> ExitCode {
    let outcome = match args().nth(1).as_deref() {
        Some("stage2") => stage2(),
        Some("system-tests") => system_tests(),
        Some("grip-braintax") => {
            return match grip_braintax() {
                Ok(()) => ExitCode::SUCCESS,
                Err(reason) => {
                    eprintln!("grip-braintax: {reason}");
                    ExitCode::FAILURE
                }
            };
        }
        _ => {
            eprintln!("usage: cargo xtask <stage2|system-tests|grip-braintax>");
            return ExitCode::FAILURE;
        }
    };
    match outcome {
        Ok(passed) => {
            println!("\n{passed} passed!");
            ExitCode::SUCCESS
        }
        Err(reason) => {
            eprintln!("\nFailed: {reason}");
            ExitCode::FAILURE
        }
    }
}

// The library's crates scored as staged, for the commit-msg hook in .githooks/:
// given the message's file, the scores are written into it, above its
// trailers; given none, they are printed.
fn grip_braintax() -> Result<(), String> {
    let scratch = temp_dir().join(format!("xtask-grip-braintax-{}", id()));
    let runner = SystemCommandRunner::new();
    let scored = LibraryScorer::new(&runner, scratch.to_string_lossy().into_owned()).score();
    let _ = remove_dir_all(&scratch);
    let block = scored.map_err(|failure| failure.to_string())?.render();
    let Some(path) = args().nth(2) else {
        println!("{block}");
        return Ok(());
    };
    let message =
        read_to_string(&path).map_err(|error| format!("could not read {path}: {error}"))?;
    write(&path, MessageScoring::new(block).applied_to(&message))
        .map_err(|error| format!("could not write {path}: {error}"))
}

fn stage2() -> Result<&'static str, String> {
    let manifest_path = workspace_manifest_path();
    let runner = SystemCommandRunner::new();
    let parser = CrapReportParser::new();
    let every_package = vec![
        String::from(CORE_PACKAGE),
        String::from(CORE_VALIDATION_PACKAGE),
        String::from(LADDER_PACKAGE),
        String::from(PROTOCOL_PACKAGE),
        String::from(PROTOCOL_VALIDATION_PACKAGE),
        String::from(SPEC_MODEL_PACKAGE),
        String::from(SYSTEM_TESTS_PACKAGE),
        String::from(XTASK_PACKAGE),
    ];
    // The library: the two crates published, and the specification's model,
    // held to the same rules though it never is.
    let library = vec![
        String::from(CORE_PACKAGE),
        String::from(PROTOCOL_PACKAGE),
        String::from(SPEC_MODEL_PACKAGE),
    ];

    // The house rules reach every package, the crate that runs the gates
    // among them.
    let stern = SternGate::new(&runner, manifest_path.clone(), every_package);
    // Second, for the same reason stern4rust is first: removing a duplicate
    // moves code between files, which changes what the gates behind it measure.
    // The library's source, what dry4rust.toml leaves in.
    let root = workspace_root();
    let dry = DryGate::new(
        &runner,
        root.to_string_lossy().into_owned(),
        String::from(DRY_MIN_NODES),
    );
    // The library: the bill paid here is the library's.
    let crap = CrapGate::new(
        &runner,
        &parser,
        manifest_path.clone(),
        library.clone(),
        String::from(CRAP_THRESHOLD),
    );
    // Every line of the library is one the protocol asks for. The ladder runs
    // the scenario tiers -- the transition matrix, the property tests, the
    // crate against the model -- so it is measured with them, the library's
    // lines the ones that count.
    let measured = vec![
        String::from(CORE_PACKAGE),
        String::from(PROTOCOL_PACKAGE),
        String::from(SPEC_MODEL_PACKAGE),
        String::from(LADDER_PACKAGE),
    ];
    let coverage = CoverageGate::new(&runner, manifest_path.clone(), measured, None);
    // faction's lines again, and the model's, over the conformance tier alone:
    // a line no vector of the specification reaches is one it does not ask for
    // (P1-ADR-NoLineTheSpecificationDoesNotAskFor). faction-protocol is not
    // the specified component, so it is measured over every tier only.
    let conformance_coverage = CoverageGate::new(
        &runner,
        manifest_path.clone(),
        vec![
            String::from(CORE_PACKAGE),
            String::from(SPEC_MODEL_PACKAGE),
            String::from(LADDER_PACKAGE),
        ],
        Some(String::from(CONFORMANCE_TIER)),
    );
    let twin = TwinGate::new(&runner, manifest_path.clone(), library.clone());
    let iceberg = IcebergGate::new(
        &runner,
        manifest_path.clone(),
        library,
        String::from(ICEBERG_THRESHOLD),
    );
    // Last: its advisories change without the code changing (see DenyGate).
    let deny = DenyGate::new(&runner, manifest_path);

    let gates: Vec<&dyn Gate> = vec![
        &stern,
        &dry,
        &crap,
        &coverage,
        &conformance_coverage,
        &twin,
        &iceberg,
        &deny,
    ];
    GateSequence::new(gates)
        .run()
        .map(|()| "faction Stage 2")
        .map_err(|failure| failure.to_string())
}

// The system tests, each in a process of its own with ports of its own, run
// in parallel by slotgate; every run's logs under logs/, the newest kept.
fn system_tests() -> Result<&'static str, String> {
    let runner = SystemCommandRunner::new();
    let root = workspace_root();
    let log_directory = RunLogDirectory::new(root.join("logs"), RUNS_TO_KEEP);
    let slotgate = SlotgateGate::new(
        &runner,
        &log_directory,
        RunStamp::new(seconds_since_epoch()).to_directory_name(),
        format!("faction system tests (max-parallel {MAX_PARALLEL})"),
        String::from(SYSTEM_TESTS_PACKAGE),
        String::from(ALL_TESTS),
        String::from(MAX_PARALLEL),
        String::from(TIMEOUT_SECS),
    );
    GateSequence::new(vec![&slotgate])
        .run()
        .map(|()| "faction system tests")
        .map_err(|failure| failure.to_string())
}

// The one clock this binary reads: the moment a run began, which names its
// log directory.
fn seconds_since_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs()
}

fn workspace_manifest_path() -> String {
    workspace_root()
        .join("Cargo.toml")
        .to_string_lossy()
        .into_owned()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one directory below the workspace root")
        .to_path_buf()
}
