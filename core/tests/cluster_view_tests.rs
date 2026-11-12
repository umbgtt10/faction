// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::cluster_view::ClusterView;
use faction::conclusion::Conclusion;
use faction::members::Members;
use faction::peer_state::PeerState;

extern crate alloc;

fn view(peer_state: PeerState) -> ClusterView {
    ClusterView::new(
        peer_state,
        true,
        vec![1, 2],
        vec![0, 1, 2],
        3,
        false,
        Members::new(vec![0, 1, 2]),
    )
}

#[test]
fn conclusion_bootstrapped_returns_bootstrapped() {
    // Arrange
    let view = view(PeerState::Bootstrapped);

    // Act
    let conclusion = view.conclusion();

    // Assert
    assert_eq!(conclusion, Some(Conclusion::Bootstrapped));
}

#[test]
fn conclusion_timed_out_returns_none() {
    // Arrange
    let view = view(PeerState::TimedOut);

    // Act
    let conclusion = view.conclusion();

    // Assert
    assert_eq!(conclusion, None);
}

#[test]
fn is_concluded_bootstrapped_returns_true() {
    // Arrange
    let view = view(PeerState::Bootstrapped);

    // Act
    let is_concluded = view.is_concluded();

    // Assert
    assert!(is_concluded);
}

#[test]
fn is_concluded_collecting_returns_false() {
    // Arrange
    let view = view(PeerState::Collecting);

    // Act
    let is_concluded = view.is_concluded();

    // Assert
    assert!(!is_concluded);
}

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
    );

    // Assert
    assert_eq!(view.peer_state(), PeerState::TimedOut);
    assert!(!view.is_pinging_completed());
    assert_eq!(view.pinging_peers(), &[4]);
    assert_eq!(view.collecting_peers(), &[5, 6]);
    assert_eq!(view.required_count(), 2);
    assert!(view.deadline_missed());
    assert_eq!(view.members().as_slice(), &[4, 5, 6]);
}
