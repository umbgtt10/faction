// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::cluster_view::ClusterView;
use faction::members::Members;
use faction::miss_count::MissCount;
use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::reconfiguration::Reconfiguration;

extern crate alloc;

#[test]
fn deadline_missed_of_a_view_whose_deadline_was_not_missed_is_false() {
    // Arrange
    let view = ClusterView::<u64>::new(
        PeerState::Collecting,
        true,
        vec![1, 2],
        vec![0, 1],
        3,
        false,
        Members::<u64>::new(vec![0, 1, 2]),
        vec![],
        vec![],
        vec![],
        None,
        Quorum::NotTracked,
        0,
        false,
    );

    // Act
    let deadline_missed = view.deadline_missed();

    // Assert
    assert!(!deadline_missed);
}

#[test]
fn new_missed_deadline_after_bootstrap_stays_bootstrapped() {
    // Arrange & Act
    let view: ClusterView = ClusterView::new(
        PeerState::Bootstrapped,
        true,
        vec![1, 2],
        vec![0, 1, 2],
        3,
        true,
        Members::<u64>::new(vec![0, 1, 2]),
        vec![],
        vec![],
        vec![],
        None,
        Quorum::Held,
        0,
        false,
    );

    // Assert
    assert_eq!(view.peer_state(), PeerState::Bootstrapped);
    assert!(view.deadline_missed());
}

#[test]
fn new_missed_deadline_after_removal_stays_removed() {
    // Arrange & Act
    let view = ClusterView::<u64>::new(
        PeerState::Removed,
        false,
        vec![],
        vec![],
        3,
        true,
        Members::<u64>::new(vec![1, 2]),
        vec![],
        vec![],
        vec![],
        None,
        Quorum::NotTracked,
        0,
        false,
    );

    // Assert
    assert_eq!(view.peer_state(), PeerState::Removed);
    assert!(view.deadline_missed());
}

#[test]
fn new_stores_every_field() {
    // Arrange & Act
    let view = ClusterView::<u64>::new(
        PeerState::Pinging,
        false,
        vec![4],
        vec![5, 6],
        2,
        true,
        Members::<u64>::new(vec![4, 5, 6]),
        vec![Reconfiguration::AdditionAcknowledged { peer_id: 7 }],
        vec![6],
        vec![MissCount {
            peer_id: 5,
            misses: 2,
        }],
        Some(5),
        Quorum::Lost,
        3,
        false,
    );

    // Assert
    assert_eq!(view.epoch(), 3);
    assert_eq!(view.peer_state(), PeerState::TimedOut);
    assert!(!view.is_pinging_completed());
    assert_eq!(view.pinging_peers(), &[4]);
    assert_eq!(view.collecting_peers(), &[5, 6]);
    assert_eq!(view.required_count(), 2);
    assert!(view.deadline_missed());
    assert_eq!(view.members().as_slice(), &[4, 5, 6]);
    assert_eq!(
        view.reconfigurations(),
        [Reconfiguration::AdditionAcknowledged { peer_id: 7 }]
    );
    assert_eq!(view.suspected(), &[6]);
    assert_eq!(
        view.misses(),
        &[MissCount {
            peer_id: 5,
            misses: 2
        }]
    );
    assert_eq!(view.last_probed(), Some(5));
    assert_eq!(view.quorum(), Quorum::Lost);
}
