// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::freshness_policy::FreshnessPolicy;
use faction::machine::Machine;
use faction::machine_config::MachineConfig;
use faction::machine_input::MachineInput;
use faction::machine_output::MachineOutput;
use faction::machine_snapshot::MachineSnapshot;
use faction::no_op_machine_observer::NoOpMachineObserver;
use faction::quorum_policy::QuorumPolicy;
use faction::PeerId;

pub struct MachineScenarioHarness {
    coordinators: Vec<Machine>,
    current_marker: u64,
}

impl MachineScenarioHarness {
    pub fn new(peer_set: Vec<PeerId>, quorum_threshold: usize, max_delay: u64) -> Self {
        let mut coordinators = Vec::new();

        for peer_id in peer_set.iter().copied() {
            let mut machine = Machine::new(
                MachineConfig::new(
                    peer_id,
                    peer_set.clone(),
                    QuorumPolicy::new(quorum_threshold),
                    FreshnessPolicy::new(max_delay),
                ),
                Box::new(NoOpMachineObserver),
            );
            let _ = machine.apply(MachineInput::ParticipationObserved {
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

    pub fn snapshot(&self, coordinator_index: usize) -> MachineSnapshot {
        self.coordinators[coordinator_index].snapshot()
    }

    pub fn apply_participation(
        &mut self,
        coordinator_index: usize,
        peer_id: PeerId,
        freshness: u64,
    ) -> Vec<MachineOutput> {
        self.coordinators[coordinator_index].apply(MachineInput::ParticipationObserved {
            peer_id,
            freshness,
            current_marker: self.current_marker,
        })
    }

    pub fn apply_ready(
        &mut self,
        coordinator_index: usize,
        peer_id: PeerId,
        freshness: u64,
    ) -> Vec<MachineOutput> {
        self.coordinators[coordinator_index].apply(MachineInput::ReadyObserved {
            peer_id,
            freshness,
            current_marker: self.current_marker,
        })
    }

    pub fn complete_local_participation(&mut self, coordinator_index: usize) -> Vec<MachineOutput> {
        self.coordinators[coordinator_index].apply(MachineInput::LocalParticipationCompleted)
    }

    pub fn expire_deadline(&mut self, coordinator_index: usize) -> Vec<MachineOutput> {
        self.coordinators[coordinator_index].apply(MachineInput::DeadlineExpired)
    }
}
