// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::io::Write;
use std::path::PathBuf;

use faction_system_tests::spawn::Spawn;
use faction_system_tests::time_ports::TimePorts;
use faction_system_tests::transport_kind::TransportKind;

use crate::draw::scenario_source::ScenarioSource;
use crate::engine::chaos_run::ChaosRun;
use crate::engine::real_engine::RealEngine;

const CASES: [(Spawn, TransportKind); 6] = [
    (Spawn::Thread, TransportKind::InMemory),
    (Spawn::Thread, TransportKind::Channels),
    (Spawn::Thread, TransportKind::Tcp),
    (Spawn::Thread, TransportKind::Grpc),
    (Spawn::Process, TransportKind::Tcp),
    (Spawn::Process, TransportKind::Grpc),
];

// The real engine's list: each seed of `seeds` played in each of the six
// spawn and transport cases the system tests run beside the task's, on the
// machine's clock, every node's log written under `root` in a folder named for
// the seed and the case -- a real run does not replay, so it is logged as it
// runs. Each run's end is written to `out`, then how many runs failed.
pub struct RealList {
    seeds: Vec<u64>,
    source: Box<dyn ScenarioSource>,
    root: PathBuf,
    time: TimePorts,
    out: Box<dyn Write>,
}

impl RealList {
    #[must_use]
    pub fn new(
        seeds: Vec<u64>,
        source: Box<dyn ScenarioSource>,
        root: PathBuf,
        time: TimePorts,
        out: Box<dyn Write>,
    ) -> Self {
        Self {
            seeds,
            source,
            root,
            time,
            out,
        }
    }

    // The list played: how many runs failed.
    pub fn run(mut self) -> u64 {
        let mut failed = 0;
        for seed in self.seeds.clone() {
            for (spawn, transport) in CASES {
                if !self.passes(seed, spawn, transport) {
                    failed += 1;
                }
            }
        }
        let runs = self.seeds.len() * CASES.len();
        let _ = writeln!(self.out, "real engine: {runs} runs, {failed} failed");
        let _ = self.out.flush();
        failed
    }

    fn passes(&mut self, seed: u64, spawn: Spawn, transport: TransportKind) -> bool {
        let case = format!("{seed}-{spawn:?}-{transport:?}").to_lowercase();
        let folder = self.root.join(&case);
        let ended = ChaosRun::new(
            self.source.scenario(seed),
            Box::new(RealEngine::new(spawn, transport, self.time.clone())),
            Some(folder.clone()),
        )
        .run();
        let _ = match &ended {
            Ok(round) => writeln!(
                self.out,
                "{case}: end state reached at round {round}; logs in {}",
                folder.display()
            ),
            Err(failure) => writeln!(self.out, "{case}: {failure}; logs in {}", folder.display()),
        };
        let _ = self.out.flush();
        ended.is_ok()
    }
}
