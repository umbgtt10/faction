// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::env::args;
use std::env::var;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::Write;
use std::io::sink;
use std::io::stdout;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use faction_chaos::draw::generated_scenarios::GeneratedScenarios;
use faction_chaos::engine::fresh_seed::FreshSeed;
use faction_chaos::engine::real_list::RealList;
use faction_chaos::engine::regressions::Regressions;
use faction_chaos::engine::replay::Replay;
use faction_chaos::engine::soak::Soak;
use faction_chaos::invocation::Invocation;
use faction_chaos::tee::Tee;
use faction_system_tests::infra::timer::real::std_monotonic_clock::StdMonotonicClock;
use faction_system_tests::infra::timer::real::system_wall_clock::SystemWallClock;
use faction_system_tests::infra::timer::real::thread_pause::ThreadPause;
use faction_system_tests::time_ports::TimePorts;
use faction_system_tests::traits::wall_clock::WallClock;

const REAL_SEEDS: u64 = 3;

// Every log of a run in one folder, logs/chaos/<run>/, named for the run the
// xtask hands down, or for the moment it began when run on its own: the
// report in chaos.log as well as on the terminal, and each seed it replays in
// a folder of its own.
fn main() -> ExitCode {
    let words: Vec<String> = args().skip(1).collect();
    let package = Path::new(env!("CARGO_MANIFEST_DIR"));
    let run = var("FACTION_LOG_RUN").unwrap_or_else(|_| {
        SystemWallClock
            .now()
            .format("%Y-%m-%d_%H-%M-%S")
            .to_string()
    });
    let folder = package
        .ancestors()
        .nth(2)
        .map_or_else(|| PathBuf::from("logs"), |workspace| workspace.join("logs"))
        .join("chaos")
        .join(run);
    let _ = create_dir_all(&folder);
    let log: Box<dyn Write> = match File::create(folder.join("chaos.log")) {
        Ok(file) => Box::new(file),
        Err(_) => Box::new(sink()),
    };
    let mut out = Tee::new(Box::new(stdout()), log);
    let ended = match Invocation::parse(&words, &|name| var(name).ok()) {
        Ok(Invocation::Replay { seed }) => {
            let replay = Replay::new(seed, folder);
            let logs = replay.folder();
            let ended = replay.run();
            let _ = match &ended {
                Ok(round) => writeln!(
                    out,
                    "seed {seed}: end state reached at round {round}; logs in {}",
                    logs.display()
                ),
                Err(failure) => writeln!(out, "seed {seed}: {failure}; logs in {}", logs.display()),
            };
            ended.is_ok()
        }
        Ok(Invocation::Soak { from, limit }) => {
            let from = from.unwrap_or_else(|| FreshSeed::new(Arc::new(SystemWallClock)).seed());
            let failed = Soak::new(
                from,
                limit,
                Box::new(GeneratedScenarios),
                Regressions::new(package.join("chaos-regressions.json")),
                folder,
                Arc::new(StdMonotonicClock::new()),
                Box::new(out),
            )
            .run();
            failed == 0
        }
        Ok(Invocation::Real) => {
            let time = TimePorts {
                clock: Arc::new(StdMonotonicClock::new()),
                pause: Arc::new(ThreadPause),
                wall_clock: Arc::new(SystemWallClock),
            };
            let seeds = (0..REAL_SEEDS).collect();
            let failed = RealList::new(
                seeds,
                Box::new(GeneratedScenarios),
                folder,
                time,
                Box::new(out),
            )
            .run();
            failed == 0
        }
        Err(usage) => {
            let _ = writeln!(out, "{usage}");
            false
        }
    };
    if ended {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
