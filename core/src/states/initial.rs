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
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::state_snapshot::StateSnapshot;
use crate::states::pinging::Pinging;

pub struct Initial;

impl StateSnapshot for Initial {
    fn state_snapshot(&self, previous: &MachineSnapshot) -> MachineSnapshot {
        previous
            .with_lifecycle_state(ReadinessLifecycleState::Phase1Active)
            .with_phase1_count(0)
            .with_phase2_count(0)
    }
}

impl MachineState for Initial {
    fn step(
        self: Box<Self>,
        input: MachineInput,
        config: &MachineConfig,
    ) -> (Vec<MachineOutput>, Box<dyn MachineState>) {
        Box::new(Pinging::new(config.peer_count())).step(input, config)
    }

    fn accept(&self, input: &MachineInput) -> bool {
        matches!(
            input,
            MachineInput::ParticipationObserved { .. } | MachineInput::ReadyObserved { .. }
        )
    }
}
