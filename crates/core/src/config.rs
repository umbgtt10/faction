// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::instance::instance_id::InstanceId;
use crate::quorum_policy::QuorumPolicy;
use crate::types::Epoch;
use crate::types::PeerId;

pub struct Config {
    instance: InstanceId,
    peers: Vec<PeerId>,
    quorum_policy: QuorumPolicy,
    suspicion_threshold: usize,
    epoch: Epoch,
    provisional: bool,
}

impl Config {
    #[must_use]
    pub fn new(
        instance: InstanceId,
        peers: Vec<PeerId>,
        quorum_policy: QuorumPolicy,
        suspicion_threshold: usize,
        epoch: Epoch,
        provisional: bool,
    ) -> Self {
        Self {
            instance,
            peers,
            quorum_policy,
            suspicion_threshold,
            epoch,
            provisional,
        }
    }

    #[must_use]
    pub const fn epoch(&self) -> Epoch {
        self.epoch
    }

    // Whether the membership the node is created with is provisional: a
    // newcomer's, offered by its coordinator, before its own addition commits.
    // The node and the incarnation the caller created it as.
    #[must_use]
    pub const fn instance(&self) -> InstanceId {
        self.instance
    }

    #[must_use]
    pub const fn is_provisional(&self) -> bool {
        self.provisional
    }

    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.instance.node
    }

    #[must_use]
    pub fn peers(&self) -> &[PeerId] {
        &self.peers
    }

    #[must_use]
    pub const fn quorum_policy(&self) -> QuorumPolicy {
        self.quorum_policy
    }

    #[must_use]
    pub fn required_count(&self) -> usize {
        self.quorum_policy.threshold()
    }

    #[must_use]
    pub const fn suspicion_threshold(&self) -> usize {
        self.suspicion_threshold
    }
}
