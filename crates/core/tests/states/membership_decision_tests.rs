// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction::quorum_policy::QuorumPolicy;
use faction::states::membership_decision::MembershipDecision;

#[test]
fn of_a_signal_returns_none() {
    // Arrange & Act
    let decision = MembershipDecision::of(&Command::ParticipationObserved { peer_id: 6 });

    // Assert
    assert_eq!(decision, None);
}

#[test]
fn of_addition_committed_returns_committed() {
    // Arrange & Act
    let decision = MembershipDecision::of(&Command::AdditionCommitted { peer_id: 7 });

    // Assert
    assert_eq!(decision, Some(MembershipDecision::Committed(7)));
}

#[test]
fn of_join_approved_returns_approved() {
    // Arrange & Act
    let decision = MembershipDecision::of(&Command::JoinApproved { peer_id: 4 });

    // Assert
    assert_eq!(decision, Some(MembershipDecision::Approved(4)));
}

#[test]
fn of_join_rejected_returns_rejected() {
    // Arrange & Act
    let decision = MembershipDecision::of(&Command::JoinRejected { peer_id: 5 });

    // Assert
    assert_eq!(decision, Some(MembershipDecision::Rejected(5)));
}

#[test]
fn of_join_requested_returns_requested() {
    // Arrange & Act
    let decision = MembershipDecision::of(&Command::JoinRequested { peer_id: 6 });

    // Assert
    assert_eq!(decision, Some(MembershipDecision::Requested(6)));
}

#[test]
fn of_removal_committed_returns_removed() {
    // Arrange & Act
    let decision = MembershipDecision::of(&Command::RemovalCommitted { peer_id: 2 });

    // Assert
    assert_eq!(decision, Some(MembershipDecision::Removed(2)));
}

#[test]
fn of_removal_requested_returns_none() {
    // Arrange & Act
    let decision = MembershipDecision::of(&Command::RemovalRequested { peer_id: 2 });

    // Assert
    assert_eq!(decision, None);
}

#[test]
fn of_required_count_changed_returns_counted() {
    // Arrange & Act
    let decision = MembershipDecision::of(&Command::RequiredCountChanged { required_count: 3 });

    // Assert
    assert_eq!(
        decision,
        Some(MembershipDecision::Counted(QuorumPolicy::new(3)))
    );
}
