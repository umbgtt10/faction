// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::states::pinging::Pinging;
use crate::machine_config::MachineConfig;
use crate::machine_input::MachineInput;
use crate::machine_output::MachineOutput;
use crate::machine_snapshot::MachineSnapshot;
use crate::machine_state::MachineState;

pub struct Initial;

impl MachineState for Initial {
    fn step(
        self: Box<Self>,
        input: MachineInput,
        config: &MachineConfig,
    ) -> (Vec<MachineOutput>, Box<dyn MachineState>) {
        Box::new(Pinging::new(config.peer_count())).step(input, config)
    }

    fn snapshot(&self, quorum_threshold: usize) -> MachineSnapshot {
        MachineSnapshot::new(
            ReadinessLifecycleState::Phase1Active,
            None,
            false,
            false,
            0,
            0,
            quorum_threshold,
        )
    }

    fn accept(&self, input: &MachineInput) -> bool {
        matches!(
            input,
            MachineInput::ParticipationObserved { .. } | MachineInput::ReadyObserved { .. }
        )
    }
}
