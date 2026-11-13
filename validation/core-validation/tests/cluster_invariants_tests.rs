// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;
use faction::cluster_view::ClusterView;
use faction::members::Members;
use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::reconfiguration::Reconfiguration;
use faction::types::PeerId;
use faction_core_validation::cluster_invariants::ClusterInvariants;

fn concluded(collecting_peers: Vec<PeerId>, required_count: usize) -> ClusterView {
    ClusterView::new(
        PeerState::Bootstrapped,
        true,
        vec![1, 2],
        collecting_peers,
        required_count,
        false,
        Members::new(vec![0, 1, 2]),
        Reconfiguration::None,
        vec![],
        vec![],
        None,
        Quorum::Held,
    )
}

extern crate alloc;

fn pinging(pinging_peers: Vec<PeerId>, collecting_peers: Vec<PeerId>) -> ClusterView {
    ClusterView::new(
        PeerState::Pinging,
        false,
        pinging_peers,
        collecting_peers,
        3,
        false,
        Members::new(vec![0, 1, 2]),
        Reconfiguration::None,
        vec![],
        vec![],
        None,
        Quorum::NotTracked,
    )
}

#[test]
fn distinct_peer_counted_twice_names_node_and_peer() {
    // Arrange
    let invariants =
        ClusterInvariants::new(vec![pinging(vec![1], vec![]), pinging(vec![2, 2], vec![])]);

    // Act
    let violations = invariants.distinct();

    // Assert
    assert_eq!(violations, vec!["node 1 counts peer 2 twice"]);
}

#[test]
fn distinct_ready_counted_twice_names_node_and_peer() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![pinging(vec![], vec![0, 1, 0])]);

    // Act
    let violations = invariants.distinct();

    // Assert
    assert_eq!(violations, vec!["node 0 counts peer 0 twice"]);
}

#[test]
fn distinct_without_repeats_returns_nothing() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![pinging(vec![1, 2], vec![0, 2])]);

    // Act
    let violations = invariants.distinct();

    // Assert
    assert!(violations.is_empty());
}

#[test]
fn membership_members_only_returns_nothing() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![pinging(vec![0, 1, 2], vec![0, 1, 2])]);

    // Act
    let violations = invariants.membership();

    // Assert
    assert!(violations.is_empty());
}

#[test]
fn membership_non_member_participation_names_node_and_peer() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![pinging(vec![1, 9], vec![])]);

    // Act
    let violations = invariants.membership();

    // Assert
    assert_eq!(violations, vec!["node 0 counts peer 9, not a member"]);
}

#[test]
fn membership_non_member_ready_names_node_and_peer() {
    // Arrange
    let invariants =
        ClusterInvariants::new(vec![pinging(vec![], vec![]), pinging(vec![], vec![7])]);

    // Act
    let violations = invariants.membership();

    // Assert
    assert_eq!(violations, vec!["node 1 counts peer 7, not a member"]);
}

#[test]
fn quorum_concluded_at_required_count_returns_nothing() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![concluded(vec![0, 1, 2], 3)]);

    // Act
    let violations = invariants.quorum();

    // Assert
    assert!(violations.is_empty());
}

#[test]
fn quorum_concluded_below_required_count_names_node_and_counts() {
    // Arrange
    let invariants =
        ClusterInvariants::new(vec![concluded(vec![0, 1, 2], 3), concluded(vec![0, 1], 3)]);

    // Act
    let violations = invariants.quorum();

    // Assert
    assert_eq!(
        violations,
        vec!["node 1 concluded with 2 of 3 confirmations"]
    );
}

#[test]
fn quorum_not_concluded_below_required_count_returns_nothing() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![pinging(vec![1], vec![0])]);

    // Act
    let violations = invariants.quorum();

    // Assert
    assert!(violations.is_empty());
}

#[test]
fn violations_gathers_every_check() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![concluded(vec![9, 9], 3)]);

    // Act
    let violations = invariants.violations();

    // Assert
    assert_eq!(
        violations,
        vec![
            "node 0 counts peer 9 twice",
            "node 0 counts peer 9, not a member",
            "node 0 counts peer 9, not a member",
            "node 0 concluded with 2 of 3 confirmations",
        ]
    );
}
