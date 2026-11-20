// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;
use faction::reconfiguration::Reconfiguration;
use faction::types::Epoch;
use faction_system_tests::cluster::Cluster;
use faction_system_tests::infra::dice::Dice;
use faction_system_tests::infra::proposal::Proposal;
use faction_system_tests::membership::Membership;
use faction_system_tests::node_snapshot::NodeSnapshot;

use crate::harness::committed_log::CommittedLog;
use crate::harness::coordinators::Coordinators;
use crate::model::change::Change;
use crate::model::planned_change::PlannedChange;
use crate::model::scenario::Scenario;

const BACKOFF_ROUNDS: u64 = 20;
const COMMIT_GAP: u64 = 3;
const DICE_SALT: u64 = 0x4C41_5945_5200_0000;
const SEED_MIX: u64 = 0x9E37_79B9_7F4A_7C15;

// Where a planned change stands with the layer above.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Progress {
    Waiting { retry_at: u64 },
    Requested { coordinator: u64 },
    Committed,
}

// The layer above the nodes, played as a consensus layer would play it. It
// asks a coordinator -- any peer no change removes, a newcomer among them
// once added -- for each planned change once it is due, and takes every
// proposal any node makes, in the order they were made. It commits them one
// at a time, numbering them, by the specification's rule alone (§17): a
// proposal no change committed since its epoch conflicts with, that a planned
// change waits for; every other it declines, and reports the decline to its
// coordinator once the coordinator holds the epoch it was declined at. It
// commits a new required count of its own accord, once the members reach it.
// A request a coordinator refused, gave up, or lost, it asks again. It
// reports to every running node the commits it lacks, in epoch order, up to
// `delivery_lag` rounds late and, at `duplicates` percent, twice: a node
// restarted, joined or taken back is caught up from the epoch it holds.
pub struct LayerAbove {
    plan: Vec<PlannedChange>,
    progress: Vec<Progress>,
    log: CommittedLog,
    coordinators: Vec<u64>,
    bounds: Vec<usize>,
    due: Vec<u64>,
    dice: Dice,
    delivery_lag: u64,
    duplicates: u8,
    next_commit: u64,
    proposals: Vec<(u64, Proposal)>,
    declines: Vec<(u64, u64, Epoch)>,
    flaw: Option<String>,
}

impl LayerAbove {
    #[must_use]
    pub fn new(scenario: &Scenario) -> Self {
        let genesis: Vec<u64> = (0..scenario.nodes as u64).collect();
        Self {
            plan: scenario.changes.clone(),
            progress: vec![Progress::Waiting { retry_at: 0 }; scenario.changes.len()],
            log: CommittedLog::new(Membership {
                peers: genesis,
                required_count: scenario.required_count,
                epoch: 0,
            }),
            coordinators: Coordinators::of(scenario),
            bounds: scenario.changes_in_flight.clone(),
            due: Vec::new(),
            dice: Dice::new(scenario.seed.wrapping_mul(SEED_MIX) ^ DICE_SALT),
            delivery_lag: scenario.delivery_lag,
            duplicates: scenario.duplicates,
            next_commit: 0,
            proposals: Vec::new(),
            declines: Vec::new(),
            flaw: None,
        }
    }

    // One round of the layer above: the proposals made taken, the requests
    // the coordinators lost given up, one change committed and the proposals
    // no longer valid declined, the changes due requested, and every running
    // node reported what it lacks.
    pub fn act(&mut self, round: u64, cluster: &Cluster) {
        let snapshots: Vec<NodeSnapshot> = (0..cluster.node_count())
            .map(|index| cluster.snapshot(index))
            .collect();
        self.due.resize(snapshots.len(), 0);
        self.collect(cluster);
        self.withdraw_lost(round, &snapshots);
        self.commit_ready(round);
        self.request_due(round, cluster, &snapshots);
        self.deliver(round, cluster, &snapshots);
    }

    // What the layer above was asked to commit that the log cannot hold: a
    // proposal the rule admitted that adds a member or removes a peer that
    // is none.
    #[must_use]
    pub fn flaw(&self) -> Option<&str> {
        self.flaw.as_deref()
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

    // The bound of the coordinator `peer_id`: 1 past the scenario's list.
    fn bound(&self, peer_id: u64) -> usize {
        usize::try_from(peer_id)
            .ok()
            .and_then(|index| self.bounds.get(index))
            .copied()
            .unwrap_or(1)
    }

    // Every proposal every node made since the last round, in the order made.
    fn collect(&mut self, cluster: &Cluster) {
        for index in 0..cluster.node_count() {
            for proposal in cluster.take_proposals(index) {
                self.proposals.push((index as u64, proposal));
            }
        }
    }

    // The planned change at `index` committed at the next epoch, unless the
    // log cannot hold it.
    fn commit(&mut self, index: usize, round: u64) {
        let change = self.plan[index].change;
        if !self.log.fits(change) {
            self.flaw = Some(format!(
                "{change:?} at epoch {}",
                self.log.commits().len() + 1
            ));
            return;
        }
        self.log.commit(change);
        self.progress[index] = Progress::Committed;
        self.next_commit = round + 1 + self.dice.below(COMMIT_GAP);
        for due in self.due.iter_mut() {
            *due = (*due).max(round + self.dice.below(self.delivery_lag + 1));
        }
    }

    // Once its round has come: the proposals in the order made, each declined
    // while it is no longer valid or no planned change waits for it, the
    // first still valid committed; with none, a new count due the members
    // reach.
    fn commit_ready(&mut self, round: u64) {
        if round < self.next_commit {
            return;
        }
        while !self.proposals.is_empty() {
            let (coordinator, proposal) = self.proposals.remove(0);
            match self.planned_for(proposal) {
                Some(index) if self.log.admits(proposal) => {
                    self.commit(index, round);
                    return;
                }
                _ => self.declines.push((
                    coordinator,
                    Self::named(proposal),
                    self.log.commits().len() as Epoch,
                )),
            }
        }
        if let Some(index) = (0..self.plan.len()).find(|index| self.count_ready(*index, round)) {
            self.commit(index, round);
        }
    }

    // A coordinator with room for another change: concluded, its changes in
    // flight and those `busy` asked of it this round fewer than its bound --
    // one holding changes already before one holding none, so that changes
    // pile up on a coordinator as far as its bound lets them.
    fn coordinator(&mut self, snapshots: &[NodeSnapshot], busy: &[u64]) -> Option<u64> {
        let held = |peer_id: u64| {
            snapshots[peer_id as usize].reconfigurations.len()
                + busy.iter().filter(|busy| **busy == peer_id).count()
        };
        let room: Vec<u64> = self
            .coordinators
            .iter()
            .copied()
            .filter(|peer_id| {
                snapshots[*peer_id as usize].peer_state == PeerState::Bootstrapped
                    && held(*peer_id) < self.bound(*peer_id)
            })
            .collect();
        let holding: Vec<u64> = room
            .iter()
            .copied()
            .filter(|peer_id| held(*peer_id) > 0)
            .collect();
        let pool = if holding.is_empty() { room } else { holding };
        if pool.is_empty() {
            return None;
        }
        Some(pool[self.dice.below(pool.len() as u64) as usize])
    }

    // Whether the planned change at `index` is a new count to commit now: its
    // round come, and the members reaching it.
    fn count_ready(&self, index: usize, round: u64) -> bool {
        matches!(
            (self.plan[index].change, self.progress[index]),
            (Change::RequiredCount { .. }, Progress::Waiting { .. })
        ) && self.is_due(index, round)
            && self.log.fits(self.plan[index].change)
    }

    // Every running node reported, in epoch order, the commits it lacks, as
    // far as its lateness lets them reach it this round; and every
    // coordinator holding the epoch a proposal of its was declined at, the
    // decline.
    fn deliver(&mut self, round: u64, cluster: &Cluster, snapshots: &[NodeSnapshot]) {
        for (index, snapshot) in snapshots.iter().enumerate() {
            if snapshot.members.is_empty() || snapshot.peer_state == PeerState::Removed {
                continue;
            }
            let mut next = self.log.after(snapshot.epoch);
            while next < self.log.commits().len() && self.due[index] <= round {
                let commit = self.log.commits()[next];
                cluster.report_commit(index as u64, commit);
                if self.dice.rolls(self.duplicates) {
                    cluster.report_commit(index as u64, commit);
                }
                next += 1;
                self.due[index] = round + self.dice.below(self.delivery_lag + 1);
            }
        }
        let (due, waiting): (Vec<_>, Vec<_>) = self
            .declines
            .iter()
            .copied()
            .partition(|(coordinator, _, epoch)| snapshots[*coordinator as usize].epoch >= *epoch);
        self.declines = waiting;
        for (coordinator, peer_id, epoch) in due {
            cluster.report_decline(coordinator, peer_id, epoch);
        }
    }

    // Whether `change` is in flight on the coordinator `snapshot` shows.
    fn in_flight(change: Change, snapshot: &NodeSnapshot) -> bool {
        match change {
            Change::Addition { peer_id } => snapshot.reconfigurations.iter().any(|change| {
                matches!(
                    change,
                    Reconfiguration::AdditionRequested { peer_id: requested }
                        | Reconfiguration::AdditionAcknowledged { peer_id: requested }
                        if *requested == peer_id
                )
            }),
            Change::Removal { peer_id } => snapshot.reconfigurations.iter().any(|change| {
                matches!(
                    change,
                    Reconfiguration::RemovalRequested { peer_id: removed, .. } if *removed == peer_id
                )
            }),
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

    // The peer a proposal names.
    fn named(proposal: Proposal) -> u64 {
        match proposal {
            Proposal::Addition { peer_id, .. } | Proposal::Removal { peer_id, .. } => peer_id,
        }
    }

    // The planned change `proposal` proposes, not yet committed.
    fn planned_for(&self, proposal: Proposal) -> Option<usize> {
        let change = match proposal {
            Proposal::Addition { peer_id, .. } => Change::Addition { peer_id },
            Proposal::Removal { peer_id, .. } => Change::Removal { peer_id },
        };
        (0..self.plan.len()).find(|index| {
            self.plan[*index].change == change && self.progress[*index] != Progress::Committed
        })
    }

    fn request(cluster: &Cluster, coordinator: u64, change: Change) {
        match change {
            Change::Addition { peer_id } => cluster.request_addition(coordinator as usize, peer_id),
            Change::Removal { peer_id } => cluster.request_removal(coordinator as usize, peer_id),
            Change::RequiredCount { .. } => {}
        }
    }

    // Every change due asked of an idle coordinator: in flight if it took it,
    // waiting to be asked again if it refused.
    fn request_due(&mut self, round: u64, cluster: &Cluster, snapshots: &[NodeSnapshot]) {
        let mut busy: Vec<u64> = Vec::new();
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
    // given up, or lost to a restart -- waiting to be asked again.
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
