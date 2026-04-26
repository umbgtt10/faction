// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::vibe_config::VibeConfig;
use crate::vibe_input::VibeInput;
use crate::vibe_output::VibeOutput;
use crate::vibe_snapshot::VibeSnapshot;
use crate::vibe_state::VibeState;

pub struct ReadyByQuorum {
    pub(super) phase1_confirmed: Vec<bool>,
    pub(super) phase2_confirmed: Vec<bool>,
    pub(super) phase1_confirmed_count: usize,
    pub(super) phase2_confirmed_count: usize,
}

impl VibeState for ReadyByQuorum {
    fn apply(
        self: Box<Self>,
        input: VibeInput,
        _config: &VibeConfig,
    ) -> (Vec<VibeOutput>, Box<dyn VibeState>) {
        let Self {
            phase1_confirmed,
            phase2_confirmed,
            phase1_confirmed_count,
            phase2_confirmed_count,
        } = *self;

        let output = match input {
            VibeInput::ParticipationObserved { peer_id, .. } => {
                vec![VibeOutput::StaleParticipationIgnored { peer_id }]
            }
            VibeInput::ReadyObserved { peer_id, .. } => {
                vec![VibeOutput::StaleReadyIgnored { peer_id }]
            }
            VibeInput::LocalParticipationCompleted => Vec::new(),
            VibeInput::DeadlineExpired => Vec::new(),
        };

        (
            output,
            Box::new(Self {
                phase1_confirmed,
                phase2_confirmed,
                phase1_confirmed_count,
                phase2_confirmed_count,
            }),
        )
    }

    fn snapshot(&self, quorum_threshold: usize) -> VibeSnapshot {
        VibeSnapshot::new(
            ReadinessLifecycleState::ReadyByQuorum,
            Some(ReadinessExitMode::Quorum),
            true,
            true,
            self.phase1_confirmed_count,
            self.phase2_confirmed_count,
            quorum_threshold,
        )
    }

    fn accepts(&self, _input: &VibeInput) -> bool {
        false
    }
}
