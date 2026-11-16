// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::engine::chaos_run::ChaosRun;
use crate::engine::shrunk::Shrunk;
use crate::model::scenario::Scenario;

// A failing scenario made as small as it still fails at, on the
// deterministic engine: its fault windows, its crashes and its changes
// dropped one at a time -- a change with the changes that wait for it --
// its commits made prompt and single, and its chaos halved, each step kept
// while the scenario still fails, until none is.
pub struct Shrinker {
    failing: Scenario,
}

impl Shrinker {
    #[must_use]
    pub fn new(failing: Scenario) -> Self {
        Self { failing }
    }

    // The smallest scenario found still failing, and its failure; nothing if
    // the scenario does not fail.
    #[must_use]
    pub fn shrink(self) -> Option<Shrunk> {
        let failure = ChaosRun::new(self.failing.clone(), None).run().err()?;
        let mut shrunk = Shrunk {
            scenario: self.failing,
            failure,
        };
        while let Some(smaller) =
            Self::candidates(&shrunk.scenario)
                .into_iter()
                .find_map(|candidate| {
                    ChaosRun::new(candidate.clone(), None)
                        .run()
                        .err()
                        .map(|failure| Shrunk {
                            scenario: candidate,
                            failure,
                        })
                })
        {
            shrunk = smaller;
        }
        Some(shrunk)
    }

    // Every scenario one step smaller than `scenario`.
    fn candidates(scenario: &Scenario) -> Vec<Scenario> {
        let faults = (0..scenario.faults.len()).map(|index| {
            let mut smaller = scenario.clone();
            smaller.faults.remove(index);
            smaller
        });
        let restarts = (0..scenario.restarts.len()).map(|index| {
            let mut smaller = scenario.clone();
            smaller.restarts.remove(index);
            smaller
        });
        let changes =
            (0..scenario.changes.len()).map(|index| Self::without_change(scenario, index));
        let prompt = (scenario.delivery_lag > 0 || scenario.duplicates > 0).then(|| Scenario {
            delivery_lag: 0,
            duplicates: 0,
            ..scenario.clone()
        });
        let shorter = (scenario.chaos_rounds > 0).then(|| Scenario {
            chaos_rounds: scenario.chaos_rounds / 2,
            ..scenario.clone()
        });
        faults
            .chain(restarts)
            .chain(changes)
            .chain(prompt)
            .chain(shorter)
            .collect()
    }

    // `scenario` without the change at `index`, nor any change waiting for it.
    fn without_change(scenario: &Scenario, index: usize) -> Scenario {
        let mut dropped = vec![index];
        while let Some(waiting) =
            scenario
                .changes
                .iter()
                .enumerate()
                .position(|(place, planned)| {
                    !dropped.contains(&place)
                        && planned.after.is_some_and(|after| dropped.contains(&after))
                })
        {
            dropped.push(waiting);
        }
        let kept: Vec<usize> = (0..scenario.changes.len())
            .filter(|place| !dropped.contains(place))
            .collect();
        let changes = kept
            .iter()
            .map(|place| {
                let mut planned = scenario.changes[*place];
                planned.after = planned
                    .after
                    .and_then(|after| kept.iter().position(|kept| *kept == after));
                planned
            })
            .collect();
        Scenario {
            changes,
            ..scenario.clone()
        }
    }
}
