// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::env::args;
use std::env::var;
use std::io::stdout;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use faction_chaos::draw::generated_scenarios::GeneratedScenarios;
use faction_chaos::engine::regressions::Regressions;
use faction_chaos::engine::replay::Replay;
use faction_chaos::engine::soak::Soak;
use faction_chaos::invocation::Invocation;
use faction_system_tests::infra::timer::real::std_monotonic_clock::StdMonotonicClock;
use faction_system_tests::infra::timer::real::system_wall_clock::SystemWallClock;
use faction_system_tests::traits::wall_clock::WallClock;

fn main() -> ExitCode {
    let words: Vec<String> = args().skip(1).collect();
    let package = Path::new(env!("CARGO_MANIFEST_DIR"));
    let logs = package.ancestors().nth(2).map_or_else(
        || PathBuf::from("logs/chaos"),
        |workspace| workspace.join("logs/chaos"),
    );
    let ended = match Invocation::parse(&words, &|name| var(name).ok()) {
        Ok(Invocation::Replay { seed }) => {
            let replay = Replay::new(seed, logs);
            let folder = replay.folder();
            let ended = replay.run();
            match &ended {
                Ok(round) => println!(
                    "seed {seed}: end state reached at round {round}; logs in {}",
                    folder.display()
                ),
                Err(failure) => println!("seed {seed}: {failure}; logs in {}", folder.display()),
            }
            ended.is_ok()
        }
        Ok(Invocation::Soak { from, limit }) => {
            let from = from.unwrap_or_else(|| SystemWallClock.now().timestamp().unsigned_abs());
            let failed = Soak::new(
                from,
                limit,
                Box::new(GeneratedScenarios),
                Regressions::new(package.join("chaos-regressions.json")),
                Arc::new(StdMonotonicClock::new()),
                Box::new(stdout()),
            )
            .run();
            failed == 0
        }
        Err(usage) => {
            eprintln!("{usage}");
            false
        }
    };
    if ended {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
