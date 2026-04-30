// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClusterView {
    lifecycle_state: ReadinessLifecycleState,
    local_participation_complete: bool,
    phase1_confirmed_count: usize,
    phase2_confirmed_count: usize,
    quorum_threshold: usize,
}

impl ClusterView {
    #[must_use]
    pub const fn new(
        lifecycle_state: ReadinessLifecycleState,
        local_participation_complete: bool,
        phase1_confirmed_count: usize,
        phase2_confirmed_count: usize,
        quorum_threshold: usize,
    ) -> Self {
        Self {
            lifecycle_state,
            local_participation_complete,
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
        match self.lifecycle_state {
            ReadinessLifecycleState::Bootstrapped => Some(ReadinessExitMode::Bootstrapped),
            ReadinessLifecycleState::TimedOut => Some(ReadinessExitMode::TimedOut),
            _ => None,
        }
    }

    #[must_use]
    pub const fn local_participation_complete(&self) -> bool {
        self.local_participation_complete
    }

    #[must_use]
    pub const fn readiness_exited(&self) -> bool {
        matches!(
            self.lifecycle_state,
            ReadinessLifecycleState::Bootstrapped | ReadinessLifecycleState::TimedOut
        )
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
    pub const fn with_local_participation_complete(mut self, val: bool) -> Self {
        self.local_participation_complete = val;
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
