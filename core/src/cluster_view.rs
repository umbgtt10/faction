// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::readiness_exit_mode::ReadinessExitMode;
use crate::node_state::NodeState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClusterView {
    node_state: NodeState,
    local_participation_complete: bool,
    pinging_confirmed_count: usize,
    collecting_confirmed_count: usize,
    required_count: usize,
}

impl ClusterView {
    #[must_use]
    pub const fn new(
        node_state: NodeState,
        local_participation_complete: bool,
        pinging_confirmed_count: usize,
        collecting_confirmed_count: usize,
        required_count: usize,
    ) -> Self {
        Self {
            node_state,
            local_participation_complete,
            pinging_confirmed_count,
            collecting_confirmed_count,
            required_count,
        }
    }

    #[must_use]
    pub const fn node_state(&self) -> NodeState {
        self.node_state
    }

    #[must_use]
    pub const fn exit_mode(&self) -> Option<ReadinessExitMode> {
        match self.node_state {
            NodeState::Bootstrapped => Some(ReadinessExitMode::Bootstrapped),
            NodeState::TimedOut => Some(ReadinessExitMode::TimedOut),
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
            self.node_state,
            NodeState::Bootstrapped | NodeState::TimedOut
        )
    }

    #[must_use]
    pub const fn pinging_confirmed_count(&self) -> usize {
        self.pinging_confirmed_count
    }

    #[must_use]
    pub const fn collecting_confirmed_count(&self) -> usize {
        self.collecting_confirmed_count
    }

    #[must_use]
    pub const fn required_count(&self) -> usize {
        self.required_count
    }

    #[must_use]
    pub const fn with_node_state(mut self, state: NodeState) -> Self {
        self.node_state = state;
        self
    }

    #[must_use]
    pub const fn with_local_participation_complete(mut self, val: bool) -> Self {
        self.local_participation_complete = val;
        self
    }

    #[must_use]
    pub const fn with_pinging_count(mut self, count: usize) -> Self {
        self.pinging_confirmed_count = count;
        self
    }

    #[must_use]
    pub const fn with_collecting_count(mut self, count: usize) -> Self {
        self.collecting_confirmed_count = count;
        self
    }
}
