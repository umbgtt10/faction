// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;

use faction_system_tests::cluster::Cluster;
use faction_system_tests::infra::fault_settings::FaultSettings;
use faction_system_tests::infra::timer::faulty::clock_fault::ClockFault;
use faction_system_tests::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;

use crate::model::fault::Fault;
use crate::model::fault_window::FaultWindow;
use crate::model::restart_from::RestartFrom;
use crate::model::restart_plan::RestartPlan;

// A scenario's faults, round by round, set on its cluster's nodes -- each
// node sent its settings when they change -- on each
// layer of a node, the fault of the last window opened over it, but that a
// timer side stays dead through every window that kills it, whatever opens
// after it, a restart alone healing it; a node down muted, deafened and its
// timer side dead; a node under head-of-line
// blocking wedged behind the peers that cannot answer -- down, mute or deaf.
// A node whose timer side died is restarted, which is what heals it; healed,
// every layer is faithful.
pub struct FaultSchedule {
    windows: Vec<FaultWindow>,
    restarts: Vec<RestartPlan>,
    peers: Vec<u64>,
    applied: BTreeMap<u64, FaultSettings>,
}

impl FaultSchedule {
    #[must_use]
    pub fn new(windows: Vec<FaultWindow>, restarts: Vec<RestartPlan>, peers: Vec<u64>) -> Self {
        Self {
            windows,
            restarts,
            peers,
            applied: BTreeMap::new(),
        }
    }

    // The faults in force at `round` set on the cluster's nodes.
    pub fn apply(&mut self, round: u64, cluster: &Cluster) {
        let down = self.down(round);
        let unreachable = self.unreachable(round, &down);
        for peer_id in self.peers.clone() {
            let settings = FaultSettings {
                transport: self.transport(peer_id, round, &down, &unreachable),
                timer: self.timer(peer_id, round, &down),
                clock: self.clock(peer_id, round),
            };
            self.set(peer_id, settings, cluster);
        }
    }

    // Every layer of every node of the cluster faithful again.
    pub fn heal(&mut self, cluster: &Cluster) {
        for peer_id in self.peers.clone() {
            self.set(peer_id, FaultSettings::default(), cluster);
        }
    }

    // The nodes a heal at `round` restarts: those down, and those whose
    // timer side is dead, at it or until it.
    #[must_use]
    pub fn healed_by_restart(&self, round: u64) -> Vec<u64> {
        let ending = self.restarts_at(round);
        self.peers
            .iter()
            .copied()
            .filter(|peer_id| {
                self.down(round).contains(peer_id)
                    || self.timer(*peer_id, round, &[]).dead
                    || ending.iter().any(|plan| plan.peer_id == *peer_id)
            })
            .collect()
    }

    // The restarts at `round`: of the nodes whose downtime ends, and of those
    // whose timer side comes back from the dead, from what they hold.
    #[must_use]
    pub fn restarts_at(&self, round: u64) -> Vec<RestartPlan> {
        let ended = self
            .restarts
            .iter()
            .copied()
            .filter(|plan| plan.at + plan.down_for == round);
        let revived = self
            .windows
            .iter()
            .filter(|window| {
                window.until == round && matches!(window.fault, Fault::Timer(policy) if policy.dead)
            })
            .map(|window| RestartPlan {
                peer_id: window.peer_id,
                at: round,
                down_for: 0,
                from: RestartFrom::Snapshot { behind: 0 },
            });
        ended.chain(revived).collect()
    }

    fn clock(&self, peer_id: u64, round: u64) -> ClockFault {
        self.latest(peer_id, round, |fault| match fault {
            Fault::Clock(fault) => Some(*fault),
            _ => None,
        })
        .unwrap_or_default()
    }

    fn down(&self, round: u64) -> Vec<u64> {
        self.restarts
            .iter()
            .filter(|plan| (plan.at..plan.at + plan.down_for).contains(&round))
            .map(|plan| plan.peer_id)
            .collect()
    }

    // The fault `pick` finds in the window last opened over `peer_id` of
    // those in force at `round`.
    fn latest<T>(&self, peer_id: u64, round: u64, pick: impl Fn(&Fault) -> Option<T>) -> Option<T> {
        self.windows
            .iter()
            .filter(|window| {
                window.peer_id == peer_id && (window.from..window.until).contains(&round)
            })
            .filter_map(|window| pick(&window.fault).map(|fault| (window.from, fault)))
            .max_by_key(|(from, _)| *from)
            .map(|(_, fault)| fault)
    }

    // The transport fault of the last window opened over `peer_id`.
    fn opened(&self, peer_id: u64, round: u64) -> FaultPolicy {
        self.latest(peer_id, round, |fault| match fault {
            Fault::Transport(policy) => Some(policy.clone()),
            _ => None,
        })
        .unwrap_or_default()
    }

    // `settings` sent to the node `peer_id`, unless they are what it holds.
    fn set(&mut self, peer_id: u64, settings: FaultSettings, cluster: &Cluster) {
        if self.applied.get(&peer_id) != Some(&settings) {
            cluster.set_faults(peer_id, &settings);
            self.applied.insert(peer_id, settings);
        }
    }

    fn timer(&self, peer_id: u64, round: u64, down: &[u64]) -> TimerFaultPolicy {
        let policy = self
            .latest(peer_id, round, |fault| match fault {
                Fault::Timer(policy) => Some(*policy),
                _ => None,
            })
            .unwrap_or_default();
        let killed = self.windows.iter().any(|window| {
            window.peer_id == peer_id
                && (window.from..window.until).contains(&round)
                && matches!(window.fault, Fault::Timer(policy) if policy.dead)
        });
        TimerFaultPolicy {
            dead: killed || down.contains(&peer_id),
            ..policy
        }
    }

    fn transport(
        &self,
        peer_id: u64,
        round: u64,
        down: &[u64],
        unreachable: &[u64],
    ) -> FaultPolicy {
        let policy = self.opened(peer_id, round);
        let is_down = down.contains(&peer_id);
        FaultPolicy {
            mute: policy.mute || is_down,
            deaf: policy.deaf || is_down,
            unreachable: if policy.head_of_line {
                unreachable.to_vec()
            } else {
                Vec::new()
            },
            ..policy
        }
    }

    // The peers no neighbour resolution reaches at `round`: down, mute or
    // deaf.
    fn unreachable(&self, round: u64, down: &[u64]) -> Vec<u64> {
        self.peers
            .iter()
            .copied()
            .filter(|peer_id| {
                let policy = self.opened(*peer_id, round);
                down.contains(peer_id) || policy.mute || policy.deaf
            })
            .collect()
    }
}
