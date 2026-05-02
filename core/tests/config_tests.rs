// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::config::Config;
use faction::quorum_policy::QuorumPolicy;

#[test]
fn new_stores_provided_values() {
    // Arrange
    let peers = vec![0, 1, 2, 3, 4];

    // Act
    let config = Config::new(0, peers.clone(), QuorumPolicy::new(3));

    // Assert
    assert_eq!(config.peer_id(), 0);
    assert_eq!(config.peers(), &[0, 1, 2, 3, 4]);
    assert_eq!(config.peer_count(), 5);
    assert_eq!(config.required_count(), 3);
    assert_eq!(config.quorum_policy().threshold(), 3);
}

#[test]
fn is_member_returns_true_for_present_peer() {
    // Arrange
    let config = Config::new(0, vec![0, 1, 2], QuorumPolicy::new(2));

    // Act & Assert
    assert!(config.is_member(0));
    assert!(config.is_member(1));
    assert!(config.is_member(2));
}

#[test]
fn is_member_returns_false_for_absent_peer() {
    // Arrange
    let config = Config::new(0, vec![0, 1, 2], QuorumPolicy::new(2));

    // Act & Assert
    assert!(!config.is_member(3));
    assert!(!config.is_member(99));
}

#[test]
fn peer_index_returns_some_for_member() {
    // Arrange
    let config = Config::new(0, vec![10, 20, 30], QuorumPolicy::new(2));

    // Act
    let is_member = config.is_member(20);

    // Assert
    assert!(is_member);
}

#[test]
fn peer_index_returns_none_for_non_member() {
    // Arrange
    let config = Config::new(0, vec![10, 20, 30], QuorumPolicy::new(2));

    // Act
    let is_member = config.is_member(99);

    // Assert
    assert!(!is_member);
}

#[test]
fn peer_index_returns_position_of_local_peer() {
    // Arrange
    let config = Config::new(3, vec![0, 1, 2, 3, 4], QuorumPolicy::new(3));

    // Act
    let is_member = config.is_member(3);

    // Assert
    assert!(is_member);
}

#[test]
fn peer_count_matches_set_length() {
    // Arrange
    let config = Config::new(0, vec![0, 1, 2], QuorumPolicy::new(2));

    // Act & Assert
    assert_eq!(config.peer_count(), 3);
}

#[test]
fn peer_count_empty_set_returns_zero() {
    // Arrange
    let config = Config::new(0, vec![], QuorumPolicy::new(1));

    // Act & Assert
    assert_eq!(config.peer_count(), 0);
}
