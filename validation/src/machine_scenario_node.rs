// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::machine::Machine;
use faction::machine_config::MachineConfig;
use faction::machine_observer::MachineObserver;
use faction::machine_output::MachineOutput;
use faction::machine_snapshot::MachineSnapshot;
use faction::no_op_machine_observer::NoOpMachineObserver;
use faction::PeerId;

pub struct MachineScenarioNode {
    peer_id: PeerId,
    readiness: Machine,
}

impl MachineScenarioNode {
    #[must_use]
    pub fn new(peer_id: PeerId, config: MachineConfig) -> Self {
        Self {
            peer_id,
            readiness: Machine::new(config, Box::new(NoOpMachineObserver)),
        }
    }

    #[must_use]
    pub fn new_with_observer(
        peer_id: PeerId,
        config: MachineConfig,
        observer: Box<dyn MachineObserver>,
    ) -> Self {
        Self {
            peer_id,
            readiness: Machine::new(config, observer),
        }
    }

    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    #[must_use]
    pub fn config(&self) -> &MachineConfig {
        self.readiness.config()
    }

    #[must_use]
    pub fn snapshot(&self) -> MachineSnapshot {
        self.readiness.snapshot()
    }

    #[must_use]
    pub fn apply(self, input: faction::machine_input::MachineInput) -> (Self, Vec<MachineOutput>) {
        let mut readiness = self.readiness;
        let outputs = readiness.apply(input);

        (
            Self {
                peer_id: self.peer_id,
                readiness,
            },
            outputs,
        )
    }
}
