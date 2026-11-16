// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use faction_system_tests::traits::monotonic_clock::MonotonicClock;

use crate::draw::scenario_source::ScenarioSource;
use crate::engine::chaos_run::ChaosRun;
use crate::engine::deterministic_engine::DeterministicEngine;
use crate::engine::regression::Regression;
use crate::engine::regressions::Regressions;
use crate::engine::shrinker::Shrinker;
use crate::engine::soak_limit::SoakLimit;

const SUMMARY_EVERY: u64 = 100;

// Seeds played one after another from `from`, on the deterministic engine,
// until the limit: each failure replayed with its nodes' logs in a folder
// under `logs` named for its seed, shrunk -- the smallest scenario replayed
// into `shrunk` beside them -- recorded among the regressions and reported as
// it is found, and a summary every hundred scenarios and at the end, written
// to `out`.
pub struct Soak {
    from: u64,
    limit: SoakLimit,
    source: Box<dyn ScenarioSource>,
    regressions: Regressions,
    logs: PathBuf,
    clock: Arc<dyn MonotonicClock>,
    out: Box<dyn Write>,
}

impl Soak {
    #[must_use]
    pub fn new(
        from: u64,
        limit: SoakLimit,
        source: Box<dyn ScenarioSource>,
        regressions: Regressions,
        logs: PathBuf,
        clock: Arc<dyn MonotonicClock>,
        out: Box<dyn Write>,
    ) -> Self {
        Self {
            from,
            limit,
            source,
            regressions,
            logs,
            clock,
            out,
        }
    }

    // The soak played: how many seeds failed.
    pub fn run(mut self) -> u64 {
        let start = self.clock.now();
        let mut played = 0;
        let mut failed = 0;
        while !self
            .limit
            .reached(played, self.clock.now().saturating_sub(start))
        {
            let seed = self.from + played;
            if !self.passes(seed) {
                failed += 1;
            }
            played += 1;
            if played % SUMMARY_EVERY == 0 {
                self.summary(played, failed);
            }
        }
        self.summary(played, failed);
        failed
    }

    // Whether `seed` passes; if not, its scenario shrunk, recorded and
    // reported.
    fn passes(&mut self, seed: u64) -> bool {
        let scenario = self.source.scenario(seed);
        let Err(failure) =
            ChaosRun::new(scenario.clone(), Box::new(DeterministicEngine::new()), None).run()
        else {
            return true;
        };
        let folder = self.logs.join(seed.to_string());
        let _ = ChaosRun::new(
            scenario.clone(),
            Box::new(DeterministicEngine::new()),
            Some(folder.clone()),
        )
        .run();
        let _ = writeln!(
            self.out,
            "seed {seed} failed: {failure}; logs in {}",
            folder.display()
        );
        let Some(shrunk) = Shrinker::new(scenario).shrink() else {
            return false;
        };
        let shrunk_folder = folder.join("shrunk");
        let _ = ChaosRun::new(
            shrunk.scenario.clone(),
            Box::new(DeterministicEngine::new()),
            Some(shrunk_folder.clone()),
        )
        .run();
        let _ = writeln!(
            self.out,
            "seed {seed} shrunk: {}; logs in {}",
            shrunk.failure,
            shrunk_folder.display()
        );
        let recorded = self.regressions.record(Regression {
            seed,
            failure: shrunk.failure.to_string(),
            scenario: shrunk.scenario,
        });
        if let Err(error) = recorded {
            let _ = writeln!(self.out, "seed {seed} not recorded: {error}");
        }
        false
    }

    fn summary(&mut self, played: u64, failed: u64) {
        let _ = writeln!(
            self.out,
            "soak from seed {}: {played} scenarios, {failed} failed",
            self.from
        );
        let _ = self.out.flush();
    }
}
