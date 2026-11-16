// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_system_tests::infra::fault_board::FaultBoard;
use faction_system_tests::infra::timer::faulty::clock_fault::ClockFault;
use faction_system_tests::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;

use crate::model::fault::Fault;
use crate::model::fault_window::FaultWindow;
use crate::model::restart_from::RestartFrom;
use crate::model::restart_plan::RestartPlan;

// A scenario's faults, round by round, on the board its nodes read: on each
// layer of a node, the fault of the last window opened over it; a node down
// muted, deafened and its timer side dead; a node under head-of-line
// blocking wedged behind the peers that cannot answer -- down, mute or deaf.
// A node whose timer side died is restarted, which is what heals it; healed,
// every layer is faithful.
pub struct FaultSchedule {
    windows: Vec<FaultWindow>,
    restarts: Vec<RestartPlan>,
    peers: Vec<PeerId>,
    board: FaultBoard,
}

impl FaultSchedule {
    #[must_use]
    pub fn new(
        windows: Vec<FaultWindow>,
        restarts: Vec<RestartPlan>,
        peers: Vec<PeerId>,
        board: FaultBoard,
    ) -> Self {
        Self {
            windows,
            restarts,
            peers,
            board,
        }
    }

    // The faults in force at `round` set on the board.
    pub fn apply(&self, round: u64) {
        let down = self.down(round);
        let unreachable = self.unreachable(round, &down);
        for peer_id in self.peers.iter().copied() {
            let faults = self.board.node(peer_id);
            faults
                .transport
                .set(self.transport(peer_id, round, &down, &unreachable));
            faults.timer.set(self.timer(peer_id, round, &down));
            faults.clock.set(self.clock(peer_id, round));
        }
    }

    // Every layer of every node faithful again.
    pub fn heal(&self) {
        for peer_id in self.peers.iter().copied() {
            let faults = self.board.node(peer_id);
            faults.transport.set(FaultPolicy::default());
            faults.timer.set(TimerFaultPolicy::default());
            faults.clock.set(ClockFault::default());
        }
    }

    // The nodes a heal at `round` restarts: those down, and those whose
    // timer side is dead, at it or until it.
    #[must_use]
    pub fn healed_by_restart(&self, round: u64) -> Vec<PeerId> {
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

    fn clock(&self, peer_id: PeerId, round: u64) -> ClockFault {
        self.latest(peer_id, round, |fault| match fault {
            Fault::Clock(fault) => Some(*fault),
            _ => None,
        })
        .unwrap_or_default()
    }

    fn down(&self, round: u64) -> Vec<PeerId> {
        self.restarts
            .iter()
            .filter(|plan| (plan.at..plan.at + plan.down_for).contains(&round))
            .map(|plan| plan.peer_id)
            .collect()
    }

    // The fault `pick` finds in the window last opened over `peer_id` of
    // those in force at `round`.
    fn latest<T>(
        &self,
        peer_id: PeerId,
        round: u64,
        pick: impl Fn(&Fault) -> Option<T>,
    ) -> Option<T> {
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
    fn opened(&self, peer_id: PeerId, round: u64) -> FaultPolicy {
        self.latest(peer_id, round, |fault| match fault {
            Fault::Transport(policy) => Some(policy.clone()),
            _ => None,
        })
        .unwrap_or_default()
    }

    fn timer(&self, peer_id: PeerId, round: u64, down: &[PeerId]) -> TimerFaultPolicy {
        let policy = self
            .latest(peer_id, round, |fault| match fault {
                Fault::Timer(policy) => Some(*policy),
                _ => None,
            })
            .unwrap_or_default();
        TimerFaultPolicy {
            dead: policy.dead || down.contains(&peer_id),
            ..policy
        }
    }

    fn transport(
        &self,
        peer_id: PeerId,
        round: u64,
        down: &[PeerId],
        unreachable: &[PeerId],
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
    fn unreachable(&self, round: u64, down: &[PeerId]) -> Vec<PeerId> {
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
