// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::machine_config::MachineConfig;
use crate::machine_input::MachineInput;
use crate::machine_output::MachineOutput;
use crate::machine_snapshot::MachineSnapshot;
use crate::machine_state::MachineState;
use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::state_snapshot::StateSnapshot;

pub struct ReadyByDeadline {
    pub phase1_count: usize,
    pub phase2_count: usize,
}

impl StateSnapshot for ReadyByDeadline {
    fn state_snapshot(&self, previous: &MachineSnapshot) -> MachineSnapshot {
        previous
            .with_lifecycle_state(ReadinessLifecycleState::ReadyByDeadline)
            .with_exit_mode(Some(ReadinessExitMode::Deadline))
            .with_readiness_exited(true)
            .with_phase1_count(self.phase1_count)
            .with_phase2_count(self.phase2_count)
    }
}

impl MachineState for ReadyByDeadline {
    fn step(
        self: Box<Self>,
        _input: MachineInput,
        _config: &MachineConfig,
    ) -> (Vec<MachineOutput>, Box<dyn MachineState>) {
        unreachable!("accept() rejects all inputs for this state")
    }

    fn accept(&self, _input: &MachineInput) -> bool {
        false
    }
}
