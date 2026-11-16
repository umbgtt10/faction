// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;
use faction::reconfiguration::Reconfiguration;
use faction::types::PeerId;
use faction_system_tests::cluster::Cluster;
use faction_system_tests::infra::dice::Dice;
use faction_system_tests::membership::Membership;
use faction_system_tests::node_snapshot::NodeSnapshot;

use crate::harness::committed_log::CommittedLog;
use crate::model::change::Change;
use crate::model::planned_change::PlannedChange;
use crate::model::scenario::Scenario;

const BACKOFF_ROUNDS: u64 = 20;
const COMMIT_GAP: u64 = 3;
const DICE_SALT: u64 = 0x4C41_5945_5200_0000;

// Where a planned change stands with the layer above.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Progress {
    Waiting { retry_at: u64 },
    Requested { coordinator: PeerId },
    Committed,
}

// The layer above the nodes, played as a consensus layer would play it. It
// asks a coordinator -- a genesis member no change removes -- for each
// planned change once it is due, commits the proposals it sees one at a
// time, numbering them, and commits a new required count of its own accord.
// A request a coordinator refused, or lost, it asks again. It reports to
// every running node the commits it lacks, in epoch order, up to
// `delivery_lag` rounds late and, at `duplicates` percent, twice: a node
// restarted, joined or taken back is caught up from the epoch it holds.
pub struct LayerAbove {
    plan: Vec<PlannedChange>,
    progress: Vec<Progress>,
    log: CommittedLog,
    coordinators: Vec<PeerId>,
    due: Vec<u64>,
    dice: Dice,
    delivery_lag: u64,
    duplicates: u8,
    next_commit: u64,
}

impl LayerAbove {
    #[must_use]
    pub fn new(scenario: &Scenario) -> Self {
        let genesis: Vec<PeerId> = (0..scenario.nodes as PeerId).collect();
        let coordinators = genesis
            .iter()
            .copied()
            .filter(|peer_id| {
                !scenario
                    .changes
                    .iter()
                    .any(|planned| planned.change == Change::Removal { peer_id: *peer_id })
            })
            .collect();
        Self {
            plan: scenario.changes.clone(),
            progress: vec![Progress::Waiting { retry_at: 0 }; scenario.changes.len()],
            log: CommittedLog::new(Membership {
                peers: genesis,
                required_count: scenario.required_count,
                epoch: 0,
            }),
            coordinators,
            due: Vec::new(),
            dice: Dice::new(scenario.seed ^ DICE_SALT),
            delivery_lag: scenario.delivery_lag,
            duplicates: scenario.duplicates,
            next_commit: 0,
        }
    }

    // One round of the layer above: the requests the coordinators lost given
    // up, one proposal committed, the changes due requested, and every
    // running node reported what it lacks.
    pub fn act(&mut self, round: u64, cluster: &Cluster) {
        let snapshots: Vec<NodeSnapshot> = (0..cluster.node_count())
            .map(|index| cluster.snapshot(index))
            .collect();
        self.due.resize(snapshots.len(), 0);
        self.withdraw_lost(round, &snapshots);
        self.commit_ready(round, &snapshots);
        self.request_due(round, cluster, &snapshots);
        self.deliver(round, cluster, &snapshots);
    }

    #[must_use]
    pub fn log(&self) -> &CommittedLog {
        &self.log
    }

    // Every commit from now on reported at once, and once: quiescence.
    pub fn quiesce(&mut self) {
        self.delivery_lag = 0;
        self.duplicates = 0;
        self.due.fill(0);
    }

    // Whether every planned change has committed.
    #[must_use]
    pub fn settled(&self) -> bool {
        self.progress
            .iter()
            .all(|progress| *progress == Progress::Committed)
    }

    fn commit_ready(&mut self, round: u64, snapshots: &[NodeSnapshot]) {
        if round < self.next_commit {
            return;
        }
        let Some(index) = (0..self.plan.len()).find(|index| self.ready(*index, round, snapshots))
        else {
            return;
        };
        self.log.commit(self.plan[index].change);
        self.progress[index] = Progress::Committed;
        self.next_commit = round + 1 + self.dice.below(COMMIT_GAP);
        for due in self.due.iter_mut() {
            *due = (*due).max(round + self.dice.below(self.delivery_lag + 1));
        }
    }

    // An idle coordinator, none of `busy`: concluded, with no change in flight.
    fn coordinator(&mut self, snapshots: &[NodeSnapshot], busy: &[PeerId]) -> Option<PeerId> {
        let idle: Vec<PeerId> = self
            .coordinators
            .iter()
            .copied()
            .filter(|peer_id| {
                let snapshot = &snapshots[*peer_id as usize];
                !busy.contains(peer_id)
                    && snapshot.peer_state == PeerState::Bootstrapped
                    && snapshot.reconfiguration == Reconfiguration::None
            })
            .collect();
        if idle.is_empty() {
            return None;
        }
        Some(idle[self.dice.below(idle.len() as u64) as usize])
    }

    // Every running node reported, in epoch order, the commits it lacks, as
    // far as its lateness lets them reach it this round.
    fn deliver(&mut self, round: u64, cluster: &Cluster, snapshots: &[NodeSnapshot]) {
        for (index, snapshot) in snapshots.iter().enumerate() {
            if snapshot.members.is_empty() || snapshot.peer_state == PeerState::Removed {
                continue;
            }
            let mut next = self.log.after(snapshot.epoch);
            while next < self.log.commits().len() && self.due[index] <= round {
                let commit = self.log.commits()[next];
                cluster.report_commit(index as PeerId, commit);
                if self.dice.rolls(self.duplicates) {
                    cluster.report_commit(index as PeerId, commit);
                }
                next += 1;
                self.due[index] = round + self.dice.below(self.delivery_lag + 1);
            }
        }
    }

    // Whether `change` is in flight on the coordinator `snapshot` shows.
    fn in_flight(change: Change, snapshot: &NodeSnapshot) -> bool {
        match change {
            Change::Addition { peer_id } => matches!(
                snapshot.reconfiguration,
                Reconfiguration::AdditionRequested { peer_id: requested }
                    | Reconfiguration::AdditionAcknowledged { peer_id: requested }
                    if requested == peer_id
            ),
            Change::Removal { peer_id } => {
                snapshot.reconfiguration == Reconfiguration::RemovalRequested { peer_id }
            }
            Change::RequiredCount { .. } => false,
        }
    }

    // Whether the planned change at `index` is due: its round come, and the
    // change it waits for committed.
    fn is_due(&self, index: usize, round: u64) -> bool {
        let planned = self.plan[index];
        planned.due <= round
            && planned
                .after
                .is_none_or(|after| self.progress[after] == Progress::Committed)
    }

    // Whether the log stays valid with `change` committed: a peer added that
    // is no member, a member removed above the required count, a count the
    // members reach.
    fn keeps_the_log_valid(&self, change: Change) -> bool {
        let membership = self.log.membership();
        match change {
            Change::Addition { peer_id } => !membership.peers.contains(&peer_id),
            Change::Removal { peer_id } => {
                membership.peers.contains(&peer_id)
                    && membership.peers.len() > membership.required_count
            }
            Change::RequiredCount { required_count } => {
                (1..=membership.peers.len()).contains(&required_count)
            }
        }
    }

    // Whether the change at `index` may commit now: its proposal seen, or,
    // for a count, its round come; and the log valid with it.
    fn ready(&self, index: usize, round: u64, snapshots: &[NodeSnapshot]) -> bool {
        let change = self.plan[index].change;
        let proposed = match (change, self.progress[index]) {
            (Change::Addition { peer_id }, Progress::Requested { coordinator }) => {
                snapshots[coordinator as usize].reconfiguration
                    == Reconfiguration::AdditionAcknowledged { peer_id }
            }
            (Change::Removal { peer_id }, Progress::Requested { coordinator }) => {
                snapshots[coordinator as usize].reconfiguration
                    == Reconfiguration::RemovalRequested { peer_id }
            }
            (Change::RequiredCount { .. }, Progress::Waiting { .. }) => self.is_due(index, round),
            _ => false,
        };
        proposed && self.keeps_the_log_valid(change)
    }

    fn request(cluster: &Cluster, coordinator: PeerId, change: Change) {
        match change {
            Change::Addition { peer_id } => cluster.request_addition(coordinator as usize, peer_id),
            Change::Removal { peer_id } => cluster.request_removal(coordinator as usize, peer_id),
            Change::RequiredCount { .. } => {}
        }
    }

    // Every change due asked of an idle coordinator: in flight if it took it,
    // waiting to be asked again if it refused.
    fn request_due(&mut self, round: u64, cluster: &Cluster, snapshots: &[NodeSnapshot]) {
        let mut busy: Vec<PeerId> = Vec::new();
        for index in 0..self.plan.len() {
            if !self.requestable(index, round) {
                continue;
            }
            let Some(coordinator) = self.coordinator(snapshots, &busy) else {
                return;
            };
            busy.push(coordinator);
            let change = self.plan[index].change;
            Self::request(cluster, coordinator, change);
            self.progress[index] =
                if Self::in_flight(change, &cluster.snapshot(coordinator as usize)) {
                    Progress::Requested { coordinator }
                } else {
                    Progress::Waiting {
                        retry_at: round + BACKOFF_ROUNDS,
                    }
                };
        }
    }

    // Whether the change at `index` is to be asked of a coordinator now.
    fn requestable(&self, index: usize, round: u64) -> bool {
        matches!(self.progress[index], Progress::Waiting { retry_at } if retry_at <= round)
            && !matches!(self.plan[index].change, Change::RequiredCount { .. })
            && self.is_due(index, round)
    }

    // Every request its coordinator no longer shows in flight -- refused late,
    // or lost to a restart -- waiting to be asked again.
    fn withdraw_lost(&mut self, round: u64, snapshots: &[NodeSnapshot]) {
        for index in 0..self.plan.len() {
            if let Progress::Requested { coordinator } = self.progress[index]
                && !Self::in_flight(self.plan[index].change, &snapshots[coordinator as usize])
            {
                self.progress[index] = Progress::Waiting {
                    retry_at: round + BACKOFF_ROUNDS,
                };
            }
        }
    }
}
