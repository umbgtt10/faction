// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;

// How the nodes a cluster runs coordinate: the suspicion threshold each is
// created with, and each one's bound of changes in flight, by its peer id --
// 1 for a peer the list does not reach.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Coordination {
    suspicion_threshold: usize,
    bounds: Vec<usize>,
}

impl Coordination {
    #[must_use]
    pub fn new(suspicion_threshold: usize, bounds: Vec<usize>) -> Self {
        Self {
            suspicion_threshold,
            bounds,
        }
    }

    // The bound of the node `peer_id`.
    #[must_use]
    pub fn bound(&self, peer_id: PeerId) -> usize {
        usize::try_from(peer_id)
            .ok()
            .and_then(|index| self.bounds.get(index))
            .copied()
            .unwrap_or(1)
    }

    // Every bound, as a process node's `--bounds` writes them: comma separated.
    #[must_use]
    pub fn listed(&self) -> String {
        self.bounds
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(",")
    }

    #[must_use]
    pub const fn suspicion_threshold(&self) -> usize {
        self.suspicion_threshold
    }
}
