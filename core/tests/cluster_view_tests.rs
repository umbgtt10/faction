// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::cluster_view_builder::ClusterViewBuilder;
use faction::conclusion::Conclusion;
use faction::members::Members;
use faction::peer_state::PeerState;

fn base() -> ClusterViewBuilder {
    ClusterViewBuilder::new()
        .with_peer_state(PeerState::Bootstrapped)
        .with_is_pinging_completed(true)
        .with_pinging_peers(vec![1, 2, 3, 4, 5])
        .with_collecting_peers(vec![1, 2, 3, 4, 5, 6, 7])
        .with_required_count(3)
}

#[test]
fn with_peer_state_updates_only_peer_state() {
    // Arrange & Act
    let result = base().with_peer_state(PeerState::Pinging).build();

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
    let result = base().with_collecting_peers(vec![99]).build();

    // Assert
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert_eq!(result.conclusion(), Some(Conclusion::Bootstrapped));
    assert!(result.is_pinging_completed());
    assert!(result.is_concluded());
    assert_eq!(result.pinging_peers(), &[1, 2, 3, 4, 5]);
    assert_eq!(result.collecting_peers(), &[99]);
    assert_eq!(result.required_count(), 3);
}

#[test]
fn with_members_is_exposed_on_the_view() {
    // Arrange & Act
    let result = base().with_members(Members::new(vec![7, 8, 9])).build();

    // Assert
    assert_eq!(result.members().as_slice(), &[7, 8, 9]);
    assert_eq!(result.members().len(), 3);
    assert!(result.members().is_member(8));
    assert!(!result.members().is_member(1));
}

#[test]
fn deadline_missed_resolves_to_timed_out_at_build() {
    // Arrange & Act
    let result = base()
        .with_peer_state(PeerState::Collecting)
        .with_deadline_missed(true)
        .build();

    // Assert
    assert_eq!(result.peer_state(), PeerState::TimedOut);
    assert!(result.deadline_missed());
    assert!(!result.is_concluded());
}
