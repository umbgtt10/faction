// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::addition::Addition;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::members::Members;
use faction::miss_count::MissCount;
use faction::peer_state::PeerState;
use faction::quorum::Quorum;

fn base() -> ClusterViewBuilder {
    ClusterViewBuilder::default()
        .with_peer_state(PeerState::Bootstrapped)
        .with_is_pinging_completed(true)
        .with_pinging_peers(vec![1, 2, 3, 4, 5])
        .with_collecting_peers(vec![1, 2, 3, 4, 5, 6, 7])
        .with_required_count(3)
}

extern crate alloc;

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
    assert_ne!(result.peer_state(), PeerState::Bootstrapped);
}

#[test]
fn from_view_copies_every_field() {
    // Arrange
    let view = base()
        .with_peer_state(PeerState::Collecting)
        .with_deadline_missed(true)
        .with_members(Members::new(vec![1, 2]))
        .with_addition(Addition::Requested { peer_id: 9 })
        .build();

    // Act
    let copy = ClusterViewBuilder::from_view(&view).build();

    // Assert
    assert_eq!(copy, view);
    assert_eq!(copy.peer_state(), PeerState::TimedOut);
    assert!(copy.deadline_missed());
    assert_eq!(copy.members().as_slice(), &[1, 2]);
    assert_eq!(copy.required_count(), 3);
    assert_eq!(copy.addition(), Addition::Requested { peer_id: 9 });
}

#[test]
fn with_addition_updates_only_the_addition() {
    // Arrange & Act
    let result = base()
        .with_addition(Addition::Acknowledged { peer_id: 8 })
        .build();

    // Assert
    assert_eq!(result.addition(), Addition::Acknowledged { peer_id: 8 });
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert_eq!(result.required_count(), 3);
}

#[test]
fn with_collecting_peers_updates_only_collecting_peers() {
    // Arrange & Act
    let result = base().with_collecting_peers(vec![99]).build();

    // Assert
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert!(result.is_pinging_completed());
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert_eq!(result.pinging_peers(), &[1, 2, 3, 4, 5]);
    assert_eq!(result.collecting_peers(), &[99]);
    assert_eq!(result.required_count(), 3);
}

#[test]
fn with_last_probed_updates_only_the_member_probed_last() {
    // Arrange & Act
    let result = base().with_last_probed(Some(2)).build();

    // Assert
    assert_eq!(result.last_probed(), Some(2));
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
}

#[test]
fn with_members_is_exposed_on_the_view() {
    // Arrange & Act
    let result = base().with_members(Members::new(vec![7, 8, 9])).build();

    // Assert
    assert_eq!(result.members().as_slice(), &[7, 8, 9]);
    assert_eq!(result.members().as_slice().len(), 3);
    assert!(result.members().is_member(8));
    assert!(!result.members().is_member(1));
}

#[test]
fn with_misses_updates_only_the_missed_periods() {
    // Arrange & Act
    let result = base()
        .with_misses(vec![MissCount {
            peer_id: 1,
            misses: 2,
        }])
        .build();

    // Assert
    assert_eq!(
        result.misses(),
        &[MissCount {
            peer_id: 1,
            misses: 2
        }]
    );
    assert_eq!(result.required_count(), 3);
}

#[test]
fn with_peer_state_updates_only_peer_state() {
    // Arrange & Act
    let result = base().with_peer_state(PeerState::Pinging).build();

    // Assert
    assert_eq!(result.peer_state(), PeerState::Pinging);
    assert_ne!(result.peer_state(), PeerState::Bootstrapped);
    assert!(result.is_pinging_completed());
    assert_ne!(result.peer_state(), PeerState::Bootstrapped);
    assert_eq!(result.pinging_peers(), &[1, 2, 3, 4, 5]);
    assert_eq!(result.collecting_peers(), &[1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(result.required_count(), 3);
}

#[test]
fn with_quorum_updates_only_the_quorum() {
    // Arrange & Act
    let result = base().with_quorum(Quorum::Lost).build();

    // Assert
    assert_eq!(result.quorum(), Quorum::Lost);
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
}

#[test]
fn with_suspected_updates_only_the_suspected_members() {
    // Arrange & Act
    let result = base().with_suspected(vec![2, 1]).build();

    // Assert
    assert_eq!(result.suspected(), &[2, 1]);
    assert_eq!(result.required_count(), 3);
}
