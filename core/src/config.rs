// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

use crate::freshness_policy::FreshnessPolicy;
use crate::quorum_policy::QuorumPolicy;
use crate::PeerId;

pub struct Config {
    local_peer_id: PeerId,
    peer_set: Vec<PeerId>,
    quorum_policy: QuorumPolicy,
    freshness_policy: FreshnessPolicy,
}

impl Config {
    #[must_use]
    pub fn new(
        local_peer_id: PeerId,
        peer_set: Vec<PeerId>,
        quorum_policy: QuorumPolicy,
        freshness_policy: FreshnessPolicy,
    ) -> Self {
        Self {
            local_peer_id,
            peer_set,
            quorum_policy,
            freshness_policy,
        }
    }

    #[must_use]
    pub const fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }

    #[must_use]
    pub fn peer_set(&self) -> &[PeerId] {
        &self.peer_set
    }

    #[must_use]
    pub fn peer_count(&self) -> usize {
        self.peer_set.len()
    }

    #[must_use]
    pub fn required_count(&self) -> usize {
        self.quorum_policy.threshold()
    }

    #[must_use]
    pub const fn quorum_policy(&self) -> QuorumPolicy {
        self.quorum_policy
    }

    #[must_use]
    pub const fn freshness_policy(&self) -> FreshnessPolicy {
        self.freshness_policy
    }

    #[must_use]
    pub fn is_member(&self, peer_id: PeerId) -> bool {
        self.peer_index(peer_id).is_some()
    }

    #[must_use]
    pub fn peer_index(&self, peer_id: PeerId) -> Option<usize> {
        self.peer_set
            .iter()
            .position(|candidate| *candidate == peer_id)
    }
}
