// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::io::Write;
use std::sync::Arc;

use faction_system_tests::traits::monotonic_clock::MonotonicClock;

use crate::draw::scenario_source::ScenarioSource;
use crate::engine::chaos_run::ChaosRun;
use crate::engine::regression::Regression;
use crate::engine::regressions::Regressions;
use crate::engine::shrinker::Shrinker;
use crate::engine::soak_limit::SoakLimit;

const SUMMARY_EVERY: u64 = 100;

// Seeds played one after another from `from`, on the deterministic engine,
// until the limit: each failure shrunk, recorded among the regressions and
// reported as it is found, and a summary every hundred scenarios and at the
// end, written to `out`.
pub struct Soak {
    from: u64,
    limit: SoakLimit,
    source: Box<dyn ScenarioSource>,
    regressions: Regressions,
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
        clock: Arc<dyn MonotonicClock>,
        out: Box<dyn Write>,
    ) -> Self {
        Self {
            from,
            limit,
            source,
            regressions,
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
        let Err(failure) = ChaosRun::new(scenario.clone(), None).run() else {
            return true;
        };
        let _ = writeln!(self.out, "seed {seed} failed: {failure}");
        let Some(shrunk) = Shrinker::new(scenario).shrink() else {
            return false;
        };
        let _ = writeln!(self.out, "seed {seed} shrunk: {}", shrunk.failure);
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
