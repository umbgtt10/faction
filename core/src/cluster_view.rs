// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

use crate::node_state::NodeState;
use crate::readiness_exit_mode::ReadinessExitMode;
use crate::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterView {
    node_state: NodeState,
    is_pinging_completed: bool,
    pinging_peers: Vec<PeerId>,
    collecting_peers: Vec<PeerId>,
    required_count: usize,
}

impl ClusterView {
    #[must_use]
    pub fn new(
        node_state: NodeState,
        is_pinging_completed: bool,
        pinging_peers: Vec<PeerId>,
        collecting_peers: Vec<PeerId>,
        required_count: usize,
    ) -> Self {
        Self {
            node_state,
            is_pinging_completed,
            pinging_peers,
            collecting_peers,
            required_count,
        }
    }

    #[must_use]
    pub fn node_state(&self) -> NodeState {
        self.node_state
    }

    #[must_use]
    pub fn exit_mode(&self) -> Option<ReadinessExitMode> {
        match self.node_state {
            NodeState::Bootstrapped => Some(ReadinessExitMode::Bootstrapped),
            NodeState::TimedOut => Some(ReadinessExitMode::TimedOut),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_pinging_completed(&self) -> bool {
        self.is_pinging_completed
    }

    #[must_use]
    pub fn readiness_exited(&self) -> bool {
        matches!(
            self.node_state,
            NodeState::Bootstrapped | NodeState::TimedOut
        )
    }

    #[must_use]
    pub fn pinging_peers(&self) -> &[PeerId] {
        &self.pinging_peers
    }

    #[must_use]
    pub fn collecting_peers(&self) -> &[PeerId] {
        &self.collecting_peers
    }

    #[must_use]
    pub fn required_count(&self) -> usize {
        self.required_count
    }

    #[must_use]
    pub fn with_node_state(mut self, state: NodeState) -> Self {
        self.node_state = state;
        self
    }

    #[must_use]
    pub fn with_is_pinging_completed(mut self, val: bool) -> Self {
        self.is_pinging_completed = val;
        self
    }

    #[must_use]
    pub fn with_pinging_peers(mut self, peers: Vec<PeerId>) -> Self {
        self.pinging_peers = peers;
        self
    }

    #[must_use]
    pub fn with_collecting_peers(mut self, peers: Vec<PeerId>) -> Self {
        self.collecting_peers = peers;
        self
    }
}
