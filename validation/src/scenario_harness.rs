// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::PeerId;

pub struct ScenarioHarness {
    coordinators: Vec<Faction>,
    current_marker: u64,
}

impl ScenarioHarness {
    pub fn new(peer_set: Vec<PeerId>, quorum_threshold: usize, max_delay: u64) -> Self {
        let mut coordinators = Vec::new();

        for peer_id in peer_set.iter().copied() {
            let mut machine = Faction::new(
                Config::new(
                    peer_id,
                    peer_set.clone(),
                    QuorumPolicy::new(quorum_threshold),
                    FreshnessPolicy::new(max_delay),
                ),
                Box::new(NoOpObserver),
            );
            let _ = machine.process(Command::ParticipationObserved {
                peer_id: PeerId::MAX,
                freshness: 0,
                current_marker: 0,
            });
            coordinators.push(machine);
        }

        Self {
            coordinators,
            current_marker: 0,
        }
    }

    pub fn current_marker(&self) -> u64 {
        self.current_marker
    }

    pub fn advance_to(&mut self, marker: u64) {
        self.current_marker = marker;
    }

    pub fn advance_by(&mut self, delta: u64) {
        self.current_marker += delta;
    }

    pub fn coordinator_count(&self) -> usize {
        self.coordinators.len()
    }

    pub fn cluster_view(&mut self, coordinator_index: usize) -> ClusterView {
        match self.coordinators[coordinator_index].process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        }
    }

    pub fn apply_participation(
        &mut self,
        coordinator_index: usize,
        peer_id: PeerId,
        freshness: u64,
    ) -> Vec<Outcome> {
        match self.coordinators[coordinator_index].process(Command::ParticipationObserved {
            peer_id,
            freshness,
            current_marker: self.current_marker,
        }) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => Vec::new(),
        }
    }

    pub fn apply_ready(
        &mut self,
        coordinator_index: usize,
        peer_id: PeerId,
        freshness: u64,
    ) -> Vec<Outcome> {
        match self.coordinators[coordinator_index].process(Command::ReadyObserved {
            peer_id,
            freshness,
            current_marker: self.current_marker,
        }) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => Vec::new(),
        }
    }

    pub fn complete_local_participation(&mut self, coordinator_index: usize) -> Vec<Outcome> {
        match self.coordinators[coordinator_index].process(Command::LocalParticipationCompleted) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => Vec::new(),
        }
    }

    pub fn expire_deadline(&mut self, coordinator_index: usize) -> Vec<Outcome> {
        match self.coordinators[coordinator_index].process(Command::DeadlineExpired) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => Vec::new(),
        }
    }
}
