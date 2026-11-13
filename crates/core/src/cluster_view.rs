// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::members::Members;
use crate::miss_count::MissCount;
use crate::peer_state::PeerState;
use crate::quorum::Quorum;
use crate::reconfiguration::Reconfiguration;
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
    reconfiguration: Reconfiguration,
    suspected: Vec<PeerId>,
    misses: Vec<MissCount>,
    last_probed: Option<PeerId>,
    quorum: Quorum,
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
        reconfiguration: Reconfiguration,
        suspected: Vec<PeerId>,
        misses: Vec<MissCount>,
        last_probed: Option<PeerId>,
        quorum: Quorum,
    ) -> Self {
        let peer_state = if deadline_missed
            && !matches!(peer_state, PeerState::Bootstrapped | PeerState::Removed)
        {
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
            reconfiguration,
            suspected,
            misses,
            last_probed,
            quorum,
        }
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
    pub fn last_probed(&self) -> Option<PeerId> {
        self.last_probed
    }

    #[must_use]
    pub fn members(&self) -> &Members {
        &self.members
    }

    #[must_use]
    pub fn misses(&self) -> &[MissCount] {
        &self.misses
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
    pub fn quorum(&self) -> Quorum {
        self.quorum
    }

    #[must_use]
    pub fn reconfiguration(&self) -> Reconfiguration {
        self.reconfiguration
    }

    #[must_use]
    pub fn required_count(&self) -> usize {
        self.required_count
    }

    #[must_use]
    pub fn suspected(&self) -> &[PeerId] {
        &self.suspected
    }
}
