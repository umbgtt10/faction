// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::vec;

use faction::cluster_view::ClusterView;
use faction::conclusion::Conclusion;
use faction::peer_state::PeerState;

fn base() -> ClusterView {
    ClusterView::new(
        PeerState::Bootstrapped,
        true,
        vec![1, 2, 3, 4, 5],
        vec![1, 2, 3, 4, 5, 6, 7],
        3,
    )
}

#[test]
fn with_peer_state_updates_only_peer_state() {
    // Arrange & Act
    let result = base().with_peer_state(PeerState::Pinging);

    // Assert
    assert_eq!(result.peer_state(), PeerState::Pinging);
    assert_eq!(result.conclusion(), None);
    assert!(result.is_pinging_completed());
    assert!(!result.is_concluded());
    assert_eq!(result.pinging_peers(), &[1, 2, 3, 4, 5]);
    assert_eq!(result.collecting_peers(), &[1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(result.required_count(), 3);
}

#[test]
fn with_collecting_peers_updates_only_collecting_peers() {
    // Arrange & Act
    let result = base().with_collecting_peers(vec![99]);

    // Assert
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert_eq!(result.conclusion(), Some(Conclusion::Bootstrapped));
    assert!(result.is_pinging_completed());
    assert!(result.is_concluded());
    assert_eq!(result.pinging_peers(), &[1, 2, 3, 4, 5]);
    assert_eq!(result.collecting_peers(), &[99]);
    assert_eq!(result.required_count(), 3);
}
