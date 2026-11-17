// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::config::Config;
use faction::instance::instance_id::InstanceId;
use faction::quorum_policy::QuorumPolicy;

extern crate alloc;

#[test]
fn changes_in_flight_is_the_bound_given() {
    // Arrange
    let config = Config::new(
        InstanceId {
            node: 0,
            incarnation: 1,
        },
        vec![0, 1],
        QuorumPolicy::new(2),
        1,
        0,
        false,
        3,
    );

    // Act
    let bound = config.changes_in_flight();

    // Assert
    assert_eq!(bound, 3);
}

#[test]
fn epoch_is_the_one_given() {
    // Arrange
    let config = Config::new(
        InstanceId {
            node: 0,
            incarnation: 1,
        },
        vec![0, 1],
        QuorumPolicy::new(2),
        1,
        6,
        false,
        1,
    );

    // Act
    let epoch = config.epoch();

    // Assert
    assert_eq!(epoch, 6);
}

#[test]
fn instance_is_the_one_given_and_its_node_the_peer_id() {
    // Arrange & Act
    let config = Config::new(
        InstanceId {
            node: 3,
            incarnation: 7,
        },
        vec![1, 2, 3],
        QuorumPolicy::new(2),
        1,
        0,
        false,
        1,
    );

    // Assert
    assert_eq!(
        config.instance(),
        InstanceId {
            node: 3,
            incarnation: 7
        }
    );
    assert_eq!(config.peer_id(), 3);
}

#[test]
fn is_provisional_is_the_one_given() {
    // Arrange
    let newcomer = Config::new(
        InstanceId {
            node: 5,
            incarnation: 1,
        },
        vec![0, 5],
        QuorumPolicy::new(1),
        1,
        2,
        true,
        1,
    );

    // Act
    let provisional = [
        newcomer.is_provisional(),
        Config::new(
            InstanceId {
                node: 0,
                incarnation: 1,
            },
            vec![0],
            QuorumPolicy::new(1),
            1,
            0,
            false,
            1,
        )
        .is_provisional(),
    ];

    // Assert
    assert_eq!(provisional, [true, false]);
}

#[test]
fn new_stores_provided_values() {
    // Arrange
    let peers = vec![0, 1, 2, 3, 4];

    // Act
    let config = Config::new(
        InstanceId {
            node: 0,
            incarnation: 1,
        },
        peers.clone(),
        QuorumPolicy::new(3),
        1,
        0,
        false,
        1,
    );

    // Assert
    assert_eq!(config.peer_id(), 0);
    assert_eq!(config.peers(), &[0, 1, 2, 3, 4]);
    assert_eq!(config.peers().len(), 5);
    assert_eq!(config.required_count(), 3);
    assert_eq!(config.quorum_policy().threshold(), 3);
}

#[test]
fn suspicion_threshold_is_the_one_given() {
    // Arrange
    let config = Config::new(
        InstanceId {
            node: 0,
            incarnation: 1,
        },
        vec![0, 1],
        QuorumPolicy::new(2),
        3,
        0,
        false,
        1,
    );

    // Act
    let threshold = config.suspicion_threshold();

    // Assert
    assert_eq!(threshold, 3);
}
