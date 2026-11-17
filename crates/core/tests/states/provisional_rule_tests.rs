// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;
use faction::command::Command;
use faction::outcome::Outcome;
use faction::states::provisional_rule::ProvisionalRule;

extern crate alloc;

fn own_addition(epoch: u64) -> Command {
    Command::AdditionCommitted { peer_id: 5, epoch }
}

fn refusal() -> Vec<Outcome> {
    vec![
        Outcome::StaleEpochIgnored {
            peer_id: 1,
            epoch: 2,
        },
        Outcome::SendMembership { peer_id: 1 },
    ]
}

#[test]
fn after_an_adopted_membership_is_not_provisional() {
    // Arrange
    let rule = ProvisionalRule::new(5, true);

    // Act
    let after = rule.after(
        &Command::MembershipReceived {
            members: vec![0, 5],
            required_count: 1,
            epoch: 4,
        },
        &[Outcome::MembershipAdopted { epoch: 4 }],
    );

    // Assert
    assert!(!after);
}

#[test]
fn after_another_peer_s_commit_is_still_provisional() {
    // Arrange
    let rule = ProvisionalRule::new(5, true);

    // Act
    let after = rule.after(
        &Command::AdditionCommitted {
            peer_id: 4,
            epoch: 4,
        },
        &[Outcome::MemberAdmitted { peer_id: 4 }],
    );

    // Assert
    assert!(after);
}

#[test]
fn after_anything_at_a_node_never_provisional_is_not_provisional() {
    // Arrange
    let rule = ProvisionalRule::new(5, false);

    // Act
    let after = rule.after(
        &own_addition(4),
        &[Outcome::DuplicateMemberIgnored { peer_id: 5 }],
    );

    // Assert
    assert!(!after);
}

#[test]
fn after_its_own_addition_held_already_is_still_provisional() {
    // Arrange
    let rule = ProvisionalRule::new(5, true);

    // Act
    let after = rule.after(&own_addition(2), &[Outcome::CommitIgnored { epoch: 2 }]);

    // Assert
    assert!(after);
}

#[test]
fn after_its_own_removal_is_not_provisional() {
    // Arrange
    let rule = ProvisionalRule::new(5, true);

    // Act
    let after = rule.after(
        &Command::RemovalCommitted {
            peer_id: 5,
            epoch: 4,
        },
        &[Outcome::MemberRemoved { peer_id: 5 }],
    );

    // Assert
    assert!(!after);
}

#[test]
fn after_the_commit_of_its_own_addition_is_not_provisional() {
    // Arrange
    let rule = ProvisionalRule::new(5, true);

    // Act
    let after = rule.after(
        &own_addition(4),
        &[Outcome::DuplicateMemberIgnored { peer_id: 5 }],
    );

    // Assert
    assert!(!after);
}

#[test]
fn answered_by_a_node_never_provisional_keeps_every_outcome() {
    // Arrange
    let rule = ProvisionalRule::new(5, false);

    // Act
    let answered = rule.answered(refusal());

    // Assert
    assert_eq!(answered, refusal());
}

#[test]
fn answered_by_a_provisional_node_keeps_a_membership_it_offers_a_newcomer() {
    // Arrange
    let rule = ProvisionalRule::new(5, true);

    // Act
    let answered = rule.answered(vec![Outcome::SendMembership { peer_id: 7 }]);

    // Assert
    assert_eq!(answered, vec![Outcome::SendMembership { peer_id: 7 }]);
}

#[test]
fn answered_by_a_provisional_node_withholds_the_membership_from_a_peer_behind() {
    // Arrange
    let rule = ProvisionalRule::new(5, true);

    // Act
    let answered = rule.answered(refusal());

    // Assert
    assert_eq!(
        answered,
        vec![Outcome::StaleEpochIgnored {
            peer_id: 1,
            epoch: 2,
        }]
    );
}
