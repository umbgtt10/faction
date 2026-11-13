// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::addition::Addition;
use faction::cluster_view::ClusterView;
use faction::members::Members;
use faction::peer_state::PeerState;

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
        Addition::None,
    );

    // Assert
    assert_eq!(view.peer_state(), PeerState::Bootstrapped);
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
        Addition::Acknowledged { peer_id: 7 },
    );

    // Assert
    assert_eq!(view.peer_state(), PeerState::TimedOut);
    assert!(!view.is_pinging_completed());
    assert_eq!(view.pinging_peers(), &[4]);
    assert_eq!(view.collecting_peers(), &[5, 6]);
    assert_eq!(view.required_count(), 2);
    assert!(view.deadline_missed());
    assert_eq!(view.members().as_slice(), &[4, 5, 6]);
    assert_eq!(view.addition(), Addition::Acknowledged { peer_id: 7 });
}
