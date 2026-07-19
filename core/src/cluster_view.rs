// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

use crate::conclusion::Conclusion;
use crate::peer_state::PeerState;
use crate::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterView {
    peer_state: PeerState,
    is_pinging_completed: bool,
    pinging_peers: Vec<PeerId>,
    collecting_peers: Vec<PeerId>,
    required_count: usize,
}

impl ClusterView {
    #[must_use]
    pub fn new(
        peer_state: PeerState,
        is_pinging_completed: bool,
        pinging_peers: Vec<PeerId>,
        collecting_peers: Vec<PeerId>,
        required_count: usize,
    ) -> Self {
        Self {
            peer_state,
            is_pinging_completed,
            pinging_peers,
            collecting_peers,
            required_count,
        }
    }

    #[must_use]
    pub fn peer_state(&self) -> PeerState {
        self.peer_state
    }

    #[must_use]
    pub fn conclusion(&self) -> Option<Conclusion> {
        match self.peer_state {
            PeerState::Bootstrapped => Some(Conclusion::Bootstrapped),
            PeerState::TimedOut => Some(Conclusion::TimedOut),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_pinging_completed(&self) -> bool {
        self.is_pinging_completed
    }

    #[must_use]
    pub fn is_concluded(&self) -> bool {
        matches!(
            self.peer_state,
            PeerState::Bootstrapped | PeerState::TimedOut
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
    pub fn with_peer_state(mut self, state: PeerState) -> Self {
        self.peer_state = state;
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
