// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachineSnapshot {
    lifecycle_state: ReadinessLifecycleState,
    exit_mode: Option<ReadinessExitMode>,
    local_participation_complete: bool,
    readiness_exited: bool,
    phase1_confirmed_count: usize,
    phase2_confirmed_count: usize,
    quorum_threshold: usize,
}

impl MachineSnapshot {
    #[must_use]
    pub const fn new(
        lifecycle_state: ReadinessLifecycleState,
        exit_mode: Option<ReadinessExitMode>,
        local_participation_complete: bool,
        readiness_exited: bool,
        phase1_confirmed_count: usize,
        phase2_confirmed_count: usize,
        quorum_threshold: usize,
    ) -> Self {
        Self {
            lifecycle_state,
            exit_mode,
            local_participation_complete,
            readiness_exited,
            phase1_confirmed_count,
            phase2_confirmed_count,
            quorum_threshold,
        }
    }

    #[must_use]
    pub const fn lifecycle_state(&self) -> ReadinessLifecycleState {
        self.lifecycle_state
    }

    #[must_use]
    pub const fn exit_mode(&self) -> Option<ReadinessExitMode> {
        self.exit_mode
    }

    #[must_use]
    pub const fn local_participation_complete(&self) -> bool {
        self.local_participation_complete
    }

    #[must_use]
    pub const fn readiness_exited(&self) -> bool {
        self.readiness_exited
    }

    #[must_use]
    pub const fn phase1_confirmed_count(&self) -> usize {
        self.phase1_confirmed_count
    }

    #[must_use]
    pub const fn phase2_confirmed_count(&self) -> usize {
        self.phase2_confirmed_count
    }

    #[must_use]
    pub const fn quorum_threshold(&self) -> usize {
        self.quorum_threshold
    }

    #[must_use]
    pub const fn with_lifecycle_state(mut self, state: ReadinessLifecycleState) -> Self {
        self.lifecycle_state = state;
        self
    }

    #[must_use]
    pub const fn with_exit_mode(mut self, mode: Option<ReadinessExitMode>) -> Self {
        self.exit_mode = mode;
        self
    }

    #[must_use]
    pub const fn with_local_participation_complete(mut self, val: bool) -> Self {
        self.local_participation_complete = val;
        self
    }

    #[must_use]
    pub const fn with_readiness_exited(mut self, val: bool) -> Self {
        self.readiness_exited = val;
        self
    }

    #[must_use]
    pub const fn with_phase1_count(mut self, count: usize) -> Self {
        self.phase1_confirmed_count = count;
        self
    }

    #[must_use]
    pub const fn with_phase2_count(mut self, count: usize) -> Self {
        self.phase2_confirmed_count = count;
        self
    }
}
