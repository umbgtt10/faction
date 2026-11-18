// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::time::Duration;

use faction_system_tests::cluster::Cluster;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::infra::timer::simulated::simulated_clock::SimulatedClock;
use faction_system_tests::infra::timer::simulated::simulated_wall_clock::SimulatedWallClock;
use faction_system_tests::infra::timer::wall_time::WallTime;
use faction_system_tests::time_ports::TimePorts;
use faction_system_tests::timer_delay::TimerDelay;

use crate::engine::chaos_engine::ChaosEngine;
use crate::model::scenario::Scenario;

const STEPS_PER_ROUND: usize = 16;

// 2026-01-01T00:00:00Z, in seconds since the epoch: where a simulated run's
// wall clock starts.
const ORIGIN: u64 = 1_767_225_600;

// The deterministic engine: task nodes over the in-memory transport, on a
// simulated clock each round advances by a timer delay, every draw from the
// scenario's seed -- so a seed replays exactly. Each round each node takes
// the inputs it has, up to a budget, until its timer and its transport have
// nothing. Its logs are stamped with the time it simulates, from the first of
// January 2026, alike on every replay.
pub struct DeterministicEngine {
    clock: Arc<SimulatedClock>,
}

impl DeterministicEngine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            clock: Arc::new(SimulatedClock::at(Duration::ZERO)),
        }
    }

    fn step(cluster: &Cluster, index: usize) {
        for _ in 0..STEPS_PER_ROUND {
            if !cluster.step(index) {
                return;
            }
        }
    }
}

impl Default for DeterministicEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ChaosEngine for DeterministicEngine {
    fn builder(&self, scenario: &Scenario) -> ClusterBuilder {
        let origin = WallTime::new(Duration::from_secs(ORIGIN));
        let time = TimePorts {
            clock: self.clock.clone(),
            pause: self.clock.clone(),
            wall_clock: Arc::new(SimulatedWallClock::new(self.clock.clone(), origin)),
        };
        ClusterBuilder::new(scenario.nodes, scenario.required_count, time)
    }

    fn round(&self, cluster: &Cluster) {
        for index in 0..cluster.node_count() {
            Self::step(cluster, index);
        }
        self.clock.advance(TimerDelay::Minimal.duration());
    }
}
