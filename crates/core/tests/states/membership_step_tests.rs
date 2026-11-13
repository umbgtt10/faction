// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::members::Members;
use faction::outcome::Outcome;
use faction::quorum_policy::QuorumPolicy;
use faction::states::membership_decision::MembershipDecision;
use faction::states::membership_step::MembershipStep;

extern crate alloc;

fn members() -> Members {
    Members::new(vec![0, 1, 2])
}

fn policy() -> QuorumPolicy {
    QuorumPolicy::new(2)
}

#[test]
fn new_committed_member_returns_duplicate_ignored() {
    // Arrange & Act
    let membership = MembershipStep::new(members(), policy(), MembershipDecision::Committed(2));

    // Assert
    assert_eq!(
        membership.outcomes(),
        &[Outcome::DuplicateMemberIgnored { peer_id: 2 }]
    );
    assert_eq!(membership.members(), &members());
}

#[test]
fn new_committed_non_member_returns_member_admitted() {
    // Arrange & Act
    let membership = MembershipStep::new(members(), policy(), MembershipDecision::Committed(7));

    // Assert
    assert_eq!(
        membership.outcomes(),
        &[Outcome::MemberAdmitted { peer_id: 7 }]
    );
    assert_eq!(membership.members(), &Members::new(vec![0, 1, 2, 7]));
    assert_eq!(membership.quorum_policy(), policy());
}

#[test]
fn new_counted_returns_required_count_set_and_the_new_count() {
    // Arrange & Act
    let membership = MembershipStep::new(
        members(),
        policy(),
        MembershipDecision::Counted(QuorumPolicy::new(3)),
    );

    // Assert
    assert_eq!(
        membership.outcomes(),
        &[Outcome::RequiredCountSet { required_count: 3 }]
    );
    assert_eq!(membership.quorum_policy(), QuorumPolicy::new(3));
    assert_eq!(membership.members(), &members());
}

#[test]
fn new_join_approved_member_returns_duplicate_ignored() {
    // Arrange & Act
    let membership = MembershipStep::new(members(), policy(), MembershipDecision::Approved(1));

    // Assert
    assert_eq!(
        membership.outcomes(),
        &[Outcome::DuplicateMemberIgnored { peer_id: 1 }]
    );
    assert_eq!(membership.members(), &members());
}

#[test]
fn new_join_approved_non_member_returns_member_admitted() {
    // Arrange & Act
    let membership = MembershipStep::new(members(), policy(), MembershipDecision::Approved(5));

    // Assert
    assert_eq!(
        membership.outcomes(),
        &[Outcome::MemberAdmitted { peer_id: 5 }]
    );
    assert_eq!(membership.members(), &Members::new(vec![0, 1, 2, 5]));
}

#[test]
fn new_join_rejected_returns_join_denied() {
    // Arrange & Act
    let membership = MembershipStep::new(members(), policy(), MembershipDecision::Rejected(5));

    // Assert
    assert_eq!(membership.outcomes(), &[Outcome::JoinDenied { peer_id: 5 }]);
    assert_eq!(membership.members(), &members());
}

#[test]
fn new_join_requested_returns_emit_join_request() {
    // Arrange & Act
    let membership = MembershipStep::new(members(), policy(), MembershipDecision::Requested(5));

    // Assert
    assert_eq!(
        membership.outcomes(),
        &[Outcome::EmitJoinRequest { peer_id: 5 }]
    );
    assert_eq!(membership.members(), &members());
}

#[test]
fn new_removed_member_returns_member_removed_and_the_members_without_it() {
    // Arrange & Act
    let membership = MembershipStep::new(members(), policy(), MembershipDecision::Removed(1));

    // Assert
    assert_eq!(
        membership.outcomes(),
        &[Outcome::MemberRemoved { peer_id: 1 }]
    );
    assert_eq!(membership.members(), &Members::new(vec![0, 2]));
    assert_eq!(membership.quorum_policy(), policy());
}

#[test]
fn new_removed_non_member_returns_non_member_ignored() {
    // Arrange & Act
    let membership = MembershipStep::new(members(), policy(), MembershipDecision::Removed(9));

    // Assert
    assert_eq!(
        membership.outcomes(),
        &[Outcome::NonMemberIgnored { peer_id: 9 }]
    );
    assert_eq!(membership.members(), &members());
}
