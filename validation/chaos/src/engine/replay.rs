// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use crate::draw::generator::Generator;
use crate::engine::chaos_run::ChaosRun;
use crate::engine::deterministic_engine::DeterministicEngine;
use crate::oracle::failure::Failure;

// A seed replayed on the deterministic engine, every node's log written in a
// folder of its own under `root`, named for the seed, with their
// consolidation: the run a failure reported, to be read.
pub struct Replay {
    seed: u64,
    root: PathBuf,
}

impl Replay {
    #[must_use]
    pub fn new(seed: u64, root: PathBuf) -> Self {
        Self { seed, root }
    }

    #[must_use]
    pub fn folder(&self) -> PathBuf {
        self.root.join(self.seed.to_string())
    }

    // The seed's scenario played again: the round its end state was reached
    // at, or how it failed.
    pub fn run(&self) -> Result<u64, Failure> {
        ChaosRun::new(
            Generator::new(self.seed).scenario(),
            Box::new(DeterministicEngine::new()),
            Some(self.folder()),
        )
        .run()
    }
}
