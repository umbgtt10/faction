// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

use crate::quorum_policy::QuorumPolicy;
use crate::types::PeerId;

pub struct Config {
    peer_id: PeerId,
    peers: Vec<PeerId>,
    quorum_policy: QuorumPolicy,
}

impl Config {
    #[must_use]
    pub fn new(peer_id: PeerId, peers: Vec<PeerId>, quorum_policy: QuorumPolicy) -> Self {
        Self {
            peer_id,
            peers,
            quorum_policy,
        }
    }

    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    #[must_use]
    pub fn peers(&self) -> &[PeerId] {
        &self.peers
    }

    #[must_use]
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    #[must_use]
    pub fn required_count(&self) -> usize {
        self.quorum_policy.threshold()
    }

    #[must_use]
    pub const fn quorum_policy(&self) -> QuorumPolicy {
        self.quorum_policy
    }

    pub fn is_member(&self, peer_id: PeerId) -> bool {
        self.peers.contains(&peer_id)
    }
}
