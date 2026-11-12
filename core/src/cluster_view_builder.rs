// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::members::Members;
use crate::peer_state::PeerState;
use crate::types::PeerId;

pub struct ClusterViewBuilder {
    peer_state: PeerState,
    is_pinging_completed: bool,
    pinging_peers: Vec<PeerId>,
    collecting_peers: Vec<PeerId>,
    required_count: usize,
    deadline_missed: bool,
    members: Members,
}

impl ClusterViewBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            peer_state: PeerState::Fresh,
            is_pinging_completed: false,
            pinging_peers: Vec::new(),
            collecting_peers: Vec::new(),
            required_count: 0,
            deadline_missed: false,
            members: Members::new(Vec::new()),
        }
    }

    #[must_use]
    pub fn from_view(view: &ClusterView) -> Self {
        Self {
            peer_state: view.peer_state(),
            is_pinging_completed: view.is_pinging_completed(),
            pinging_peers: view.pinging_peers().to_vec(),
            collecting_peers: view.collecting_peers().to_vec(),
            required_count: view.required_count(),
            deadline_missed: view.deadline_missed(),
            members: view.members().clone(),
        }
    }

    #[must_use]
    pub fn build(self) -> ClusterView {
        ClusterView::new(
            self.peer_state,
            self.is_pinging_completed,
            self.pinging_peers,
            self.collecting_peers,
            self.required_count,
            self.deadline_missed,
            self.members,
        )
    }

    #[must_use]
    pub fn with_collecting_peers(mut self, peers: Vec<PeerId>) -> Self {
        self.collecting_peers = peers;
        self
    }

    #[must_use]
    pub fn with_deadline_missed(mut self, value: bool) -> Self {
        self.deadline_missed = value;
        self
    }

    #[must_use]
    pub fn with_is_pinging_completed(mut self, value: bool) -> Self {
        self.is_pinging_completed = value;
        self
    }

    #[must_use]
    pub fn with_members(mut self, members: Members) -> Self {
        self.members = members;
        self
    }

    #[must_use]
    pub fn with_peer_state(mut self, peer_state: PeerState) -> Self {
        self.peer_state = peer_state;
        self
    }

    #[must_use]
    pub fn with_pinging_peers(mut self, peers: Vec<PeerId>) -> Self {
        self.pinging_peers = peers;
        self
    }

    #[must_use]
    pub fn with_required_count(mut self, required_count: usize) -> Self {
        self.required_count = required_count;
        self
    }
}

impl Default for ClusterViewBuilder {
    fn default() -> Self {
        Self::new()
    }
}
