// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::states::phase1::Phase1;
use crate::vibe_config::VibeConfig;
use crate::vibe_input::VibeInput;
use crate::vibe_output::VibeOutput;
use crate::vibe_snapshot::VibeSnapshot;
use crate::vibe_state::VibeState;

pub struct Initial;

impl VibeState for Initial {
    fn punch(
        self: Box<Self>,
        input: VibeInput,
        config: &VibeConfig,
    ) -> (Vec<VibeOutput>, Box<dyn VibeState>) {
        Box::new(Phase1::new(config.peer_count())).punch(input, config)
    }

    fn vibe_check(&self, quorum_threshold: usize) -> VibeSnapshot {
        VibeSnapshot::new(
            ReadinessLifecycleState::Phase1Active,
            None,
            false,
            false,
            0,
            0,
            quorum_threshold,
        )
    }

    fn deal(&self, input: &VibeInput) -> bool {
        matches!(
            input,
            VibeInput::ParticipationObserved { .. } | VibeInput::ReadyObserved { .. }
        )
    }
}
