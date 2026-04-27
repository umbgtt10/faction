// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::vibe_config::VibeConfig;
use crate::vibe_input::VibeInput;
use crate::vibe_output::VibeOutput;
use crate::vibe_snapshot::VibeSnapshot;
use crate::vibe_state::VibeState;

use super::helpers::confirmed_set::ConfirmedSet;

pub struct ReadyByQuorum {
    pub phase1: ConfirmedSet,
    pub phase2: ConfirmedSet,
}

impl VibeState for ReadyByQuorum {
    fn punch(
        self: Box<Self>,
        _input: VibeInput,
        _config: &VibeConfig,
    ) -> (Vec<VibeOutput>, Box<dyn VibeState>) {
        unreachable!("deal() rejects all inputs for this state")
    }

    fn vibe_check(&self, quorum_threshold: usize) -> VibeSnapshot {
        VibeSnapshot::new(
            ReadinessLifecycleState::ReadyByQuorum,
            Some(ReadinessExitMode::Quorum),
            true,
            true,
            self.phase1.count(),
            self.phase2.count(),
            quorum_threshold,
        )
    }

    fn deal(&self, _input: &VibeInput) -> bool {
        false
    }
}
