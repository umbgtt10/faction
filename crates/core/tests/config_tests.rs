// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::config::Config;
use faction::quorum_policy::QuorumPolicy;

extern crate alloc;

#[test]
fn new_stores_provided_values() {
    // Arrange
    let peers = vec![0, 1, 2, 3, 4];

    // Act
    let config = Config::new(0, peers.clone(), QuorumPolicy::new(3));

    // Assert
    assert_eq!(config.peer_id(), 0);
    assert_eq!(config.peers(), &[0, 1, 2, 3, 4]);
    assert_eq!(config.peers().len(), 5);
    assert_eq!(config.required_count(), 3);
    assert_eq!(config.quorum_policy().threshold(), 3);
}
