// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::instance::peer_id::PeerId;
use crate::members::Members;
use crate::miss_count::MissCount;
use crate::peer_state::PeerState;
use crate::quorum::Quorum;
use crate::reconfiguration::Reconfiguration;
use crate::types::Epoch;

pub struct ClusterViewBuilder<P: PeerId = u64> {
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

impl<P: PeerId> ClusterViewBuilder<P> {
    #[must_use]
    pub fn from_view(view: &ClusterView<P>) -> Self {
        Self::from_view_at(view, view.epoch(), view.is_provisional())
    }

    // A view's builder, the view moved to `epoch`, its membership
    // `provisional` or not (§16).
    #[must_use]
    pub fn from_view_at(view: &ClusterView<P>, epoch: Epoch, provisional: bool) -> Self {
        Self {
            peer_state: view.peer_state(),
            is_pinging_completed: view.is_pinging_completed(),
            pinging_peers: view.pinging_peers().to_vec(),
            collecting_peers: view.collecting_peers().to_vec(),
            required_count: view.required_count(),
            deadline_missed: view.deadline_missed(),
            members: view.members().clone(),
            reconfigurations: view.reconfigurations().to_vec(),
            suspected: view.suspected().to_vec(),
            misses: view.misses().to_vec(),
            last_probed: view.last_probed(),
            quorum: view.quorum(),
            epoch,
            provisional,
        }
    }

    #[must_use]
    pub fn build(self) -> ClusterView<P> {
        ClusterView::new(
            self.peer_state,
            self.is_pinging_completed,
            self.pinging_peers,
            self.collecting_peers,
            self.required_count,
            self.deadline_missed,
            self.members,
            self.reconfigurations,
            self.suspected,
            self.misses,
            self.last_probed,
            self.quorum,
            self.epoch,
            self.provisional,
        )
    }

    #[must_use]
    pub fn with_collecting_peers(mut self, peers: Vec<P>) -> Self {
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
    pub fn with_last_probed(mut self, last_probed: Option<P>) -> Self {
        self.last_probed = last_probed;
        self
    }

    #[must_use]
    pub fn with_members(mut self, members: Members<P>) -> Self {
        self.members = members;
        self
    }

    #[must_use]
    pub fn with_misses(mut self, misses: Vec<MissCount<P>>) -> Self {
        self.misses = misses;
        self
    }

    #[must_use]
    pub fn with_peer_state(mut self, peer_state: PeerState) -> Self {
        self.peer_state = peer_state;
        self
    }

    #[must_use]
    pub fn with_pinging_peers(mut self, peers: Vec<P>) -> Self {
        self.pinging_peers = peers;
        self
    }

    #[must_use]
    pub fn with_quorum(mut self, quorum: Quorum) -> Self {
        self.quorum = quorum;
        self
    }

    #[must_use]
    pub fn with_reconfigurations(mut self, reconfigurations: Vec<Reconfiguration<P>>) -> Self {
        self.reconfigurations = reconfigurations;
        self
    }

    #[must_use]
    pub fn with_required_count(mut self, required_count: usize) -> Self {
        self.required_count = required_count;
        self
    }

    #[must_use]
    pub fn with_suspected(mut self, suspected: Vec<P>) -> Self {
        self.suspected = suspected;
        self
    }
}

impl<P: PeerId> Default for ClusterViewBuilder<P> {
    fn default() -> Self {
        Self {
            peer_state: PeerState::Fresh,
            is_pinging_completed: false,
            pinging_peers: Vec::new(),
            collecting_peers: Vec::new(),
            required_count: 0,
            deadline_missed: false,
            members: Members::new(Vec::new()),
            reconfigurations: Vec::new(),
            suspected: Vec::new(),
            misses: Vec::new(),
            last_probed: None,
            quorum: Quorum::NotTracked,
            epoch: 0,
            provisional: false,
        }
    }
}
