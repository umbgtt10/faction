// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use faction::freshness_policy::FreshnessPolicy;
use faction::machine::Machine;
use faction::machine_config::MachineConfig;
use faction::machine_input::MachineInput;
use faction::machine_output::MachineOutput;
use faction::machine_snapshot::MachineSnapshot;
use faction::no_op_machine_observer::NoOpMachineObserver;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::PeerId;

pub struct ClusterSimulation {
    nodes: Vec<Machine>,
    peer_ids: Vec<PeerId>,
    current_marker: u64,
    pending: VecDeque<(usize, MachineInput)>,
}

impl ClusterSimulation {
    #[must_use]
    pub fn new(peer_count: usize, quorum_threshold: usize, max_delay: u64) -> Self {
        let peer_ids: Vec<PeerId> = (0..peer_count as PeerId).collect();
        let nodes = peer_ids
            .iter()
            .map(|&peer_id| {
                Machine::new(
                    MachineConfig::new(
                        peer_id,
                        peer_ids.clone(),
                        QuorumPolicy::new(quorum_threshold),
                        FreshnessPolicy::new(max_delay),
                    ),
                    Box::new(NoOpMachineObserver),
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

    fn apply_to(&mut self, index: usize, input: MachineInput) -> Vec<MachineOutput> {
        self.nodes[index].apply(input)
    }

    fn enqueue_broadcasts(&mut self, outputs: &[MachineOutput], source_index: usize) {
        for output in outputs {
            if let MachineOutput::BroadcastLocalReady = output {
                for target in 0..self.nodes.len() {
                    if target != source_index {
                        self.pending.push_back((
                            target,
                            MachineInput::ReadyObserved {
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

    fn apply_and_drain(&mut self, index: usize, input: MachineInput) {
        let outputs = self.apply_to(index, input);
        self.enqueue_broadcasts(&outputs, index);
        self.drain_pending();
    }

    pub fn inject_participation(&mut self, peer_id: PeerId, freshness: u64) {
        for index in 0..self.nodes.len() {
            let outputs = self.apply_to(
                index,
                MachineInput::ParticipationObserved {
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
                MachineInput::ReadyObserved {
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
        self.apply_and_drain(index, MachineInput::LocalParticipationCompleted);
    }

    pub fn expire_deadline(&mut self, peer_id: PeerId) {
        let index = self
            .peer_ids
            .iter()
            .position(|p| *p == peer_id)
            .expect("peer is in the cluster");
        self.apply_and_drain(index, MachineInput::DeadlineExpired);
    }

    #[must_use]
    pub fn all_exited(&self) -> bool {
        self.nodes.iter().all(|n| n.snapshot().readiness_exited())
    }

    #[must_use]
    pub fn all_exited_with(&self, mode: ReadinessExitMode) -> bool {
        self.nodes
            .iter()
            .all(|n| n.snapshot().exit_mode() == Some(mode))
    }

    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn snapshot(&self, peer_id: PeerId) -> MachineSnapshot {
        let index = self
            .peer_ids
            .iter()
            .position(|p| *p == peer_id)
            .expect("peer is in the cluster");
        self.nodes[index].snapshot()
    }
}
