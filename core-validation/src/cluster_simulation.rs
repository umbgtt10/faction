// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use faction::process_result::ProcessResult;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::quorum_policy::QuorumPolicy;
use faction::types::PeerId;

pub struct ClusterSimulation {
    peers: Vec<Faction>,
    peer_ids: Vec<PeerId>,
    pending: VecDeque<(usize, Command)>,
}

impl ClusterSimulation {
    #[must_use]
    pub fn new(peer_count: usize, required_count: usize) -> Self {
        let peer_ids: Vec<PeerId> = (0..peer_count as PeerId).collect();
        let peers = peer_ids
            .iter()
            .map(|&peer_id| {
                Faction::new(
                    Config::new(peer_id, peer_ids.clone(), QuorumPolicy::new(required_count)),
                    Box::new(NoOpObserver),
                )
            })
            .collect();

        Self {
            peers,
            peer_ids,
            pending: VecDeque::new(),
        }
    }

    fn apply_to(&mut self, index: usize, command: Command) -> Vec<Outcome> {
        match self.peers[index].process(command) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => Vec::new(),
        }
    }

    fn enqueue_broadcasts(&mut self, outputs: &[Outcome], source_index: usize) {
        let broadcast = outputs
            .iter()
            .any(|o| matches!(o, Outcome::BroadcastLocalReady));
        if !broadcast {
            return;
        }
        for target in 0..self.peers.len() {
            if target == source_index {
                continue;
            }
            self.pending.push_back((
                target,
                Command::ReadyObserved {
                    peer_id: self.peer_ids[source_index],
                },
            ));
        }
    }

    fn drain_pending(&mut self) {
        while let Some((target, command)) = self.pending.pop_front() {
            let outputs = self.apply_to(target, command);
            self.enqueue_broadcasts(&outputs, target);
        }
    }

    fn apply_and_drain(&mut self, index: usize, command: Command) {
        let outputs = self.apply_to(index, command);
        self.enqueue_broadcasts(&outputs, index);
        self.drain_pending();
    }

    pub fn inject_participation(&mut self, peer_id: PeerId) {
        for index in 0..self.peers.len() {
            let outputs = self.apply_to(index, Command::ParticipationObserved { peer_id });
            self.enqueue_broadcasts(&outputs, index);
        }
        self.drain_pending();
    }

    pub fn inject_ready(&mut self, peer_id: PeerId) {
        for index in 0..self.peers.len() {
            let outputs = self.apply_to(index, Command::ReadyObserved { peer_id });
            self.enqueue_broadcasts(&outputs, index);
        }
        self.drain_pending();
    }

    pub fn complete_local(&mut self, peer_id: PeerId) {
        let index = self
            .peer_ids
            .iter()
            .position(|p| *p == peer_id)
            .expect("peer is in the cluster");
        self.apply_and_drain(index, Command::LocalParticipationCompleted);
    }

    pub fn expire_deadline(&mut self, peer_id: PeerId) {
        let index = self
            .peer_ids
            .iter()
            .position(|p| *p == peer_id)
            .expect("peer is in the cluster");
        self.apply_and_drain(index, Command::DeadlineExpired);
    }

    #[must_use]
    pub fn all_exited(&mut self) -> bool {
        self.peers
            .iter_mut()
            .all(|n| match n.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view.is_concluded(),
                _ => unreachable!(),
            })
    }

    #[must_use]
    pub fn all_exited_with(&mut self, mode: Conclusion) -> bool {
        self.peers
            .iter_mut()
            .all(|n| match n.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => {
                    cluster_view.conclusion() == Some(mode)
                }
                _ => unreachable!(),
            })
    }

    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn cluster_view(&mut self, peer_id: PeerId) -> ClusterView {
        let index = self
            .peer_ids
            .iter()
            .position(|p| *p == peer_id)
            .expect("peer is in the cluster");
        match self.peers[index].process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        }
    }
}
