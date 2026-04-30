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
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::PeerId;

pub struct ClusterSimulation {
    nodes: Vec<Faction>,
    peer_ids: Vec<PeerId>,
    current_marker: u64,
    pending: VecDeque<(usize, Command)>,
}

impl ClusterSimulation {
    #[must_use]
    pub fn new(peer_count: usize, required_count: usize, max_delay: u64) -> Self {
        let peer_ids: Vec<PeerId> = (0..peer_count as PeerId).collect();
        let nodes = peer_ids
            .iter()
            .map(|&peer_id| {
                Faction::new(
                    Config::new(
                        peer_id,
                        peer_ids.clone(),
                        QuorumPolicy::new(required_count),
                        FreshnessPolicy::new(max_delay),
                    ),
                    Box::new(NoOpObserver),
                )
            })
            .collect();

        Self {
            nodes,
            peer_ids,
            current_marker: 0,
            pending: VecDeque::new(),
        }
    }

    pub fn advance_to(&mut self, marker: u64) {
        self.current_marker = marker;
    }

    fn apply_to(&mut self, index: usize, input: Command) -> Vec<Outcome> {
        match self.nodes[index].process(input) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => Vec::new(),
        }
    }

    fn enqueue_broadcasts(&mut self, outputs: &[Outcome], source_index: usize) {
        for output in outputs {
            if let Outcome::BroadcastLocalReady = output {
                for target in 0..self.nodes.len() {
                    if target != source_index {
                        self.pending.push_back((
                            target,
                            Command::ReadyObserved {
                                peer_id: self.peer_ids[source_index],
                                freshness: self.current_marker,
                                current_marker: self.current_marker,
                            },
                        ));
                    }
                }
            }
        }
    }

    fn drain_pending(&mut self) {
        while let Some((target, input)) = self.pending.pop_front() {
            let outputs = self.apply_to(target, input);
            self.enqueue_broadcasts(&outputs, target);
        }
    }

    fn apply_and_drain(&mut self, index: usize, input: Command) {
        let outputs = self.apply_to(index, input);
        self.enqueue_broadcasts(&outputs, index);
        self.drain_pending();
    }

    pub fn inject_participation(&mut self, peer_id: PeerId, freshness: u64) {
        for index in 0..self.nodes.len() {
            let outputs = self.apply_to(
                index,
                Command::ParticipationObserved {
                    peer_id,
                    freshness,
                    current_marker: self.current_marker,
                },
            );
            self.enqueue_broadcasts(&outputs, index);
        }
        self.drain_pending();
    }

    pub fn inject_ready(&mut self, peer_id: PeerId, freshness: u64) {
        for index in 0..self.nodes.len() {
            let outputs = self.apply_to(
                index,
                Command::ReadyObserved {
                    peer_id,
                    freshness,
                    current_marker: self.current_marker,
                },
            );
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
        self.nodes
            .iter_mut()
            .all(|n| match n.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view.readiness_exited(),
                _ => unreachable!(),
            })
    }

    #[must_use]
    pub fn all_exited_with(&mut self, mode: ReadinessExitMode) -> bool {
        self.nodes
            .iter_mut()
            .all(|n| match n.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => {
                    cluster_view.exit_mode() == Some(mode)
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
        match self.nodes[index].process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        }
    }
}
