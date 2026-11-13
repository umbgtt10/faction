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
fn new_missed_deadline_after_bootstrap_stays_bootstrapped() {
    // Arrange & Act
    let view = ClusterView::new(
        PeerState::Bootstrapped,
        true,
        vec![1, 2],
        vec![0, 1, 2],
        3,
        true,
        Members::new(vec![0, 1, 2]),
        Reconfiguration::None,
        vec![],
        vec![],
        None,
        Quorum::Held,
    );

    // Assert
    assert_eq!(view.peer_state(), PeerState::Bootstrapped);
    assert!(view.deadline_missed());
}

#[test]
fn new_missed_deadline_after_removal_stays_removed() {
    // Arrange & Act
    let view = ClusterView::new(
        PeerState::Removed,
        false,
        vec![],
        vec![],
        3,
        true,
        Members::new(vec![1, 2]),
        Reconfiguration::None,
        vec![],
        vec![],
        None,
        Quorum::NotTracked,
    );

    // Assert
    assert_eq!(view.peer_state(), PeerState::Removed);
    assert!(view.deadline_missed());
}

#[test]
fn new_stores_every_field() {
    // Arrange & Act
    let view = ClusterView::new(
        PeerState::Pinging,
        false,
        vec![4],
        vec![5, 6],
        2,
        true,
        Members::new(vec![4, 5, 6]),
        Reconfiguration::AdditionAcknowledged { peer_id: 7 },
        vec![6],
        vec![MissCount {
            peer_id: 5,
            misses: 2,
        }],
        Some(5),
        Quorum::Lost,
    );

    // Assert
    assert_eq!(view.peer_state(), PeerState::TimedOut);
    assert!(!view.is_pinging_completed());
    assert_eq!(view.pinging_peers(), &[4]);
    assert_eq!(view.collecting_peers(), &[5, 6]);
    assert_eq!(view.required_count(), 2);
    assert!(view.deadline_missed());
    assert_eq!(view.members().as_slice(), &[4, 5, 6]);
    assert_eq!(
        view.reconfiguration(),
        Reconfiguration::AdditionAcknowledged { peer_id: 7 }
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
