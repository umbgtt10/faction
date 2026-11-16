// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::time::Duration;

use faction::types::PeerId;
use faction_system_tests::cluster::Cluster;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::infra::fault_board::FaultBoard;
use faction_system_tests::infra::invariants::breaches::Breaches;
use faction_system_tests::infra::timer::real::system_wall_clock::SystemWallClock;
use faction_system_tests::infra::timer::simulated::simulated_clock::SimulatedClock;
use faction_system_tests::time_ports::TimePorts;
use faction_system_tests::timer_delay::TimerDelay;

use crate::engine::fault_schedule::FaultSchedule;
use crate::harness::layer_above::LayerAbove;
use crate::model::change::Change;
use crate::model::restart_from::RestartFrom;
use crate::model::restart_plan::RestartPlan;
use crate::model::scenario::Scenario;
use crate::oracle::expected_end::ExpectedEnd;
use crate::oracle::failure::Failure;

const STEPS_PER_ROUND: usize = 16;

// A scenario played on the deterministic engine: task nodes over the
// in-memory transport, on a simulated clock each round advances by a timer
// delay, every fault and every draw from the scenario's seed -- so a seed
// replays exactly. Each round the faults in force are set, the layer above
// acts, and each node takes the inputs it has, up to a budget. Once the
// chaos ends every fault heals -- a node down, or with its timer side dead,
// restarted -- and the run waits for the end state the committed log
// implies, every invariant watched throughout. It announces no fault it
// injects: a seed that fails is replayed to be read.
pub struct ChaosRun {
    scenario: Scenario,
    cluster: Cluster,
    clock: Arc<SimulatedClock>,
    breaches: Breaches,
    schedule: FaultSchedule,
    layer: LayerAbove,
}

impl ChaosRun {
    #[must_use]
    pub fn new(scenario: Scenario) -> Self {
        let clock = Arc::new(SimulatedClock::at(Duration::ZERO));
        let time = TimePorts {
            clock: clock.clone(),
            pause: clock.clone(),
            wall_clock: Arc::new(SystemWallClock),
        };
        let board = FaultBoard::default();
        let breaches = Breaches::default();
        let mut cluster = ClusterBuilder::new(scenario.nodes, scenario.required_count, time)
            .suspicion_threshold(scenario.suspicion_threshold)
            .faults(board.clone())
            .breaches(breaches.clone())
            .silent()
            .build();
        cluster.start_all();
        let peers = Self::peers(&scenario);
        for peer_id in peers[scenario.nodes..].iter().copied() {
            cluster.add_newcomer(peer_id);
        }
        let schedule = FaultSchedule::new(
            scenario.faults.clone(),
            scenario.restarts.clone(),
            peers,
            board,
        );
        let layer = LayerAbove::new(&scenario);
        Self {
            scenario,
            cluster,
            clock,
            breaches,
            schedule,
            layer,
        }
    }

    // The scenario played: the round its end state was reached at, or how it
    // failed.
    pub fn run(mut self) -> Result<u64, Failure> {
        let deadline = self.scenario.chaos_rounds + self.scenario.quiescence_rounds;
        for round in 0..=deadline {
            self.disturb(round);
            self.layer.act(round, &self.cluster);
            for index in 0..self.cluster.node_count() {
                self.step(index);
            }
            self.clock.advance(TimerDelay::Minimal.duration());
            if let Some(breach) = self.breaches.first() {
                return Err(Failure::Breach { round, breach });
            }
            if round >= self.scenario.chaos_rounds
                && self.layer.settled()
                && self.departure().is_none()
            {
                return Ok(round);
            }
        }
        Err(self.failure(deadline))
    }

    // The first node short of the end state, and what it was expected to show.
    fn departure(&self) -> Option<(PeerId, String)> {
        let expected = ExpectedEnd::new(self.layer.log());
        (0..self.cluster.node_count()).find_map(|index| {
            let peer_id = index as PeerId;
            expected
                .departure(peer_id, &self.cluster.snapshot(index))
                .map(|shown| (peer_id, shown))
        })
    }

    // The chaos of `round`: its faults set and its restarts made; at its end,
    // every fault healed and the commits reported at once.
    fn disturb(&mut self, round: u64) {
        if round < self.scenario.chaos_rounds {
            self.schedule.apply(round);
            for plan in self.schedule.restarts_at(round) {
                self.restart(plan);
            }
        } else if round == self.scenario.chaos_rounds {
            self.schedule.heal();
            for peer_id in self.schedule.healed_by_restart(round) {
                self.restart(RestartPlan {
                    peer_id,
                    at: round,
                    down_for: 0,
                    from: RestartFrom::Snapshot { behind: 0 },
                });
            }
            self.layer.quiesce();
        }
    }

    fn failure(&self, round: u64) -> Failure {
        match self.departure() {
            Some((peer_id, expected)) => Failure::Diverged {
                round,
                peer_id,
                expected,
                found: Box::new(self.cluster.snapshot(peer_id as usize)),
            },
            None => Failure::Uncommitted { round },
        }
    }

    // Every peer the scenario runs: the genesis peers and the newcomers its
    // changes add, numbered on from them.
    fn peers(scenario: &Scenario) -> Vec<PeerId> {
        let last = scenario
            .changes
            .iter()
            .filter_map(|planned| match planned.change {
                Change::Addition { peer_id } => Some(peer_id + 1),
                _ => None,
            })
            .max()
            .unwrap_or(0)
            .max(scenario.nodes as PeerId);
        (0..last).collect()
    }

    // A node created again from its genesis, or from a membership of the log
    // `behind` commits short of what it holds.
    fn restart(&self, plan: RestartPlan) {
        let log = self.layer.log();
        let held = log.after(self.cluster.snapshot(plan.peer_id as usize).epoch);
        let prefix = match plan.from {
            RestartFrom::Genesis => 0,
            RestartFrom::Snapshot { behind } => held.saturating_sub(behind),
        };
        self.cluster
            .restart(plan.peer_id, log.membership_at(prefix));
    }

    // The inputs the node at `index` has taken, up to the round's budget: it
    // stops once both its timer and its transport have nothing.
    fn step(&self, index: usize) {
        let mut idle = 0;
        for _ in 0..STEPS_PER_ROUND {
            idle = if self.cluster.step(index) {
                0
            } else {
                idle + 1
            };
            if idle == 2 {
                return;
            }
        }
    }
}
