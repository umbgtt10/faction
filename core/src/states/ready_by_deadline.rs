// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;

use alloc::vec::Vec;

use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::machine_config::MachineConfig;
use crate::machine_input::MachineInput;
use crate::machine_output::MachineOutput;
use crate::machine_snapshot::MachineSnapshot;
use crate::machine_state::MachineState;

use super::helpers::confirmed_set::ConfirmedSet;

pub struct ReadyByDeadline {
    pub phase1: ConfirmedSet,
    pub phase2: ConfirmedSet,
    pub local_participation_complete: bool,
}

impl MachineState for ReadyByDeadline {
    fn step(
        self: Box<Self>,
        _input: MachineInput,
        _config: &MachineConfig,
    ) -> (Vec<MachineOutput>, Box<dyn MachineState>) {
        unreachable!("accept() rejects all inputs for this state")
    }

    fn snapshot(&self, quorum_threshold: usize) -> MachineSnapshot {
        MachineSnapshot::new(
            ReadinessLifecycleState::ReadyByDeadline,
            Some(ReadinessExitMode::Deadline),
            self.local_participation_complete,
            true,
            self.phase1.count(),
            self.phase2.count(),
            quorum_threshold,
        )
    }

    fn accept(&self, _input: &MachineInput) -> bool {
        false
    }
}
