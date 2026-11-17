// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

use faction_system_tests::cluster::Cluster;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::time_ports::TimePorts;
use faction_system_tests::timer_delay::TimerDelay;
use faction_system_tests::transport_kind::TransportKind;

use crate::engine::chaos_engine::ChaosEngine;
use crate::model::scenario::Scenario;

const SPARE_ROUNDS: u64 = 100;

// The real engine: thread or process nodes over the transport of a case, on
// the machine's clock, every node running on its own -- a seed fixes the
// scenario, not the scheduling. A round is the spawn's timer delay, waited
// out, and each node lives through every round the scenario may take, and
// a hundred more.
pub struct RealEngine {
    spawn: Spawn,
    transport: TransportKind,
    time: TimePorts,
}

impl RealEngine {
    #[must_use]
    pub fn new(spawn: Spawn, transport: TransportKind, time: TimePorts) -> Self {
        Self {
            spawn,
            transport,
            time,
        }
    }

    // A round: the timer delay of the spawn.
    fn delay(&self) -> Duration {
        match self.spawn {
            Spawn::Task => TimerDelay::Minimal,
            Spawn::Thread => TimerDelay::Moderate,
            Spawn::Process => TimerDelay::Generous,
        }
        .duration()
    }
}

impl ChaosEngine for RealEngine {
    fn builder(&self, scenario: &Scenario) -> ClusterBuilder {
        let rounds = scenario.chaos_rounds + scenario.quiescence_rounds + SPARE_ROUNDS;
        ClusterBuilder::new(scenario.nodes, scenario.required_count, self.time.clone())
            .spawn(self.spawn)
            .transport(self.transport)
            .lifetime(self.delay() * u32::try_from(rounds).unwrap_or(u32::MAX))
    }

    fn round(&self, _: &Cluster) {
        self.time.pause.pause(self.delay());
    }
}
