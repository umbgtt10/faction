// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::instance::instance_id::InstanceId;
use crate::instance::peer_id::PeerId;
use crate::quorum_policy::QuorumPolicy;
use crate::types::Epoch;

pub struct Config<P: PeerId = u64> {
    instance: InstanceId<P>,
    peers: Vec<P>,
    quorum_policy: QuorumPolicy,
    suspicion_threshold: usize,
    epoch: Epoch,
    provisional: bool,
    changes_in_flight: usize,
}

impl<P: PeerId> Config<P> {
    #[must_use]
    pub fn new(
        instance: InstanceId<P>,
        peers: Vec<P>,
        quorum_policy: QuorumPolicy,
        suspicion_threshold: usize,
        epoch: Epoch,
        provisional: bool,
        changes_in_flight: usize,
    ) -> Self {
        Self {
            instance,
            peers,
            quorum_policy,
            suspicion_threshold,
            epoch,
            provisional,
            changes_in_flight,
        }
    }

    // How many changes the node may hold in flight at once as a coordinator:
    // its bound (§2).
    #[must_use]
    pub const fn changes_in_flight(&self) -> usize {
        self.changes_in_flight
    }

    #[must_use]
    pub const fn epoch(&self) -> Epoch {
        self.epoch
    }

    // Whether the membership the node is created with is provisional: a
    // newcomer's, offered by its coordinator, before its own addition commits.
    // The node and the incarnation the caller created it as.
    #[must_use]
    pub const fn instance(&self) -> InstanceId<P> {
        self.instance
    }

    #[must_use]
    pub const fn is_provisional(&self) -> bool {
        self.provisional
    }

    #[must_use]
    pub const fn peer_id(&self) -> P {
        self.instance.node
    }

    #[must_use]
    pub fn peers(&self) -> &[P] {
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
