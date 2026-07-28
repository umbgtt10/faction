// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::conclusion::Conclusion;
use crate::members::Members;
use crate::peer_state::PeerState;
use crate::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterView {
    pub peer_state: PeerState,
    pub is_pinging_completed: bool,
    pub pinging_peers: Vec<PeerId>,
    pub collecting_peers: Vec<PeerId>,
    pub required_count: usize,
    pub deadline_missed: bool,
    pub members: Members,
}

impl ClusterView {
    #[must_use]
    pub fn peer_state(&self) -> PeerState {
        self.peer_state
    }

    #[must_use]
    pub fn conclusion(&self) -> Option<Conclusion> {
        match self.peer_state {
            PeerState::Bootstrapped => Some(Conclusion::Bootstrapped),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_pinging_completed(&self) -> bool {
        self.is_pinging_completed
    }

    #[must_use]
    pub fn is_concluded(&self) -> bool {
        matches!(self.peer_state, PeerState::Bootstrapped)
    }

    #[must_use]
    pub fn deadline_missed(&self) -> bool {
        self.deadline_missed
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
    pub fn members(&self) -> &Members {
        &self.members
    }
}
