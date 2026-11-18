// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::instance::peer_id::PeerId;
use crate::members::Members;
use crate::miss_count::MissCount;
use crate::peer_state::PeerState;
use crate::quorum::Quorum;
use crate::reconfiguration::Reconfiguration;
use crate::types::Epoch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterView<P: PeerId = u64> {
    peer_state: PeerState,
    is_pinging_completed: bool,
    pinging_peers: Vec<P>,
    collecting_peers: Vec<P>,
    required_count: usize,
    deadline_missed: bool,
    members: Members<P>,
    reconfigurations: Vec<Reconfiguration<P>>,
    suspected: Vec<P>,
    misses: Vec<MissCount<P>>,
    last_probed: Option<P>,
    quorum: Quorum,
    epoch: Epoch,
    provisional: bool,
}

impl<P: PeerId> ClusterView<P> {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        peer_state: PeerState,
        is_pinging_completed: bool,
        pinging_peers: Vec<P>,
        collecting_peers: Vec<P>,
        required_count: usize,
        deadline_missed: bool,
        members: Members<P>,
        reconfigurations: Vec<Reconfiguration<P>>,
        suspected: Vec<P>,
        misses: Vec<MissCount<P>>,
        last_probed: Option<P>,
        quorum: Quorum,
        epoch: Epoch,
        provisional: bool,
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
            reconfigurations,
            suspected,
            misses,
            last_probed,
            quorum,
            epoch,
            provisional,
        }
    }

    #[must_use]
    pub fn collecting_peers(&self) -> &[P] {
        &self.collecting_peers
    }

    #[must_use]
    pub fn deadline_missed(&self) -> bool {
        self.deadline_missed
    }

    #[must_use]
    pub fn epoch(&self) -> Epoch {
        self.epoch
    }

    #[must_use]
    pub fn is_pinging_completed(&self) -> bool {
        self.is_pinging_completed
    }

    // Whether the node's membership is provisional (§16).
    #[must_use]
    pub fn is_provisional(&self) -> bool {
        self.provisional
    }

    #[must_use]
    pub fn last_probed(&self) -> Option<P> {
        self.last_probed
    }

    #[must_use]
    pub fn members(&self) -> &Members<P> {
        &self.members
    }

    #[must_use]
    pub fn misses(&self) -> &[MissCount<P>] {
        &self.misses
    }

    #[must_use]
    pub fn peer_state(&self) -> PeerState {
        self.peer_state
    }

    #[must_use]
    pub fn pinging_peers(&self) -> &[P] {
        &self.pinging_peers
    }

    #[must_use]
    pub fn quorum(&self) -> Quorum {
        self.quorum
    }

    #[must_use]
    pub fn reconfigurations(&self) -> &[Reconfiguration<P>] {
        &self.reconfigurations
    }

    #[must_use]
    pub fn required_count(&self) -> usize {
        self.required_count
    }

    #[must_use]
    pub fn suspected(&self) -> &[P] {
        &self.suspected
    }
}
