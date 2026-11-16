// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::env::args;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use faction_chaos::engine::replay::Replay;
use faction_chaos::invocation::Invocation;

fn main() -> ExitCode {
    let words: Vec<String> = args().skip(1).collect();
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2);
    let root = workspace.map_or_else(
        || PathBuf::from("logs/chaos"),
        |root| root.join("logs/chaos"),
    );
    match Invocation::parse(&words) {
        Ok(Invocation::Replay { seed }) => {
            let replay = Replay::new(seed, root);
            let folder = replay.folder();
            match replay.run() {
                Ok(round) => {
                    println!(
                        "seed {seed}: end state reached at round {round}; logs in {}",
                        folder.display()
                    );
                    ExitCode::SUCCESS
                }
                Err(failure) => {
                    println!("seed {seed}: {failure}; logs in {}", folder.display());
                    ExitCode::FAILURE
                }
            }
        }
        Err(usage) => {
            eprintln!("{usage}");
            ExitCode::FAILURE
        }
    }
}
