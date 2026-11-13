// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::addition::Addition;
use crate::members::Members;
use crate::peer_state::PeerState;
use crate::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterView {
    peer_state: PeerState,
    is_pinging_completed: bool,
    pinging_peers: Vec<PeerId>,
    collecting_peers: Vec<PeerId>,
    required_count: usize,
    deadline_missed: bool,
    members: Members,
    addition: Addition,
}

impl ClusterView {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        peer_state: PeerState,
        is_pinging_completed: bool,
        pinging_peers: Vec<PeerId>,
        collecting_peers: Vec<PeerId>,
        required_count: usize,
        deadline_missed: bool,
        members: Members,
        addition: Addition,
    ) -> Self {
        let peer_state = if deadline_missed && peer_state != PeerState::Bootstrapped {
            PeerState::TimedOut
        } else {
            peer_state
        };

        Self {
            peer_state,
            is_pinging_completed,
            pinging_peers,
            collecting_peers,
            required_count,
            deadline_missed,
            members,
            addition,
        }
    }

    #[must_use]
    pub fn addition(&self) -> Addition {
        self.addition
    }

    #[must_use]
    pub fn collecting_peers(&self) -> &[PeerId] {
        &self.collecting_peers
    }

    #[must_use]
    pub fn deadline_missed(&self) -> bool {
        self.deadline_missed
    }

    #[must_use]
    pub fn is_pinging_completed(&self) -> bool {
        self.is_pinging_completed
    }

    #[must_use]
    pub fn members(&self) -> &Members {
        &self.members
    }

    #[must_use]
    pub fn peer_state(&self) -> PeerState {
        self.peer_state
    }

    #[must_use]
    pub fn pinging_peers(&self) -> &[PeerId] {
        &self.pinging_peers
    }

    #[must_use]
    pub fn required_count(&self) -> usize {
        self.required_count
    }
}
