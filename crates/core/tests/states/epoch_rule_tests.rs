// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::command::Command;
use faction::members::Members;
use faction::outcome::Outcome;
use faction::states::epoch_rule::EpochRule;

extern crate alloc;

fn membership(epoch: u64) -> Command {
    Command::MembershipReceived {
        members: vec![0, 1],
        required_count: 1,
        epoch,
    }
}

#[test]
fn after_of_a_claim_leaves_the_epoch() {
    // Arrange
    let rule = EpochRule::new(1);

    // Act
    let after = rule.after::<u64>(&Command::ParticipationObserved {
        peer_id: 2,
        epoch: 7,
    });

    // Assert
    assert_eq!(after, 1);
}

#[test]
fn after_of_a_commit_or_a_membership_is_the_greater_epoch() {
    // Arrange
    let rule = EpochRule::new(3);

    // Act
    let after = [
        rule.after::<u64>(&Command::AdditionCommitted {
            peer_id: 4,
            epoch: 5,
        }),
        rule.after::<u64>(&Command::RemovalCommitted {
            peer_id: 4,
            epoch: 1,
        }),
        rule.after::<u64>(&Command::RequiredCountChanged {
            required_count: 2,
            epoch: 0,
        }),
        rule.after::<u64>(&membership(4)),
    ];

    // Assert
    assert_eq!(after, [5, 3, 3, 4]);
}

#[test]
fn held_of_a_claim_is_none() {
    // Arrange
    let rule = EpochRule::new(2);

    // Act
    let held = rule.held::<u64>(&Command::ReadyObserved {
        peer_id: 1,
        epoch: 0,
    });

    // Assert
    assert_eq!(held, None);
}

#[test]
fn held_of_a_commit_from_1_to_the_epoch_is_ignored() {
    // Arrange
    let rule = EpochRule::new(2);
    let commits = [0, 1, 2, 3].map(|epoch| Command::AdditionCommitted { peer_id: 4, epoch });

    // Act
    let held = commits.map(|commit| rule.held::<u64>(&commit));

    // Assert
    assert_eq!(
        held,
        [
            None,
            Some(Outcome::CommitIgnored { epoch: 1 }),
            Some(Outcome::CommitIgnored { epoch: 2 }),
            None,
        ]
    );
}

#[test]
fn held_of_a_membership_at_or_below_the_epoch_is_ignored() {
    // Arrange
    let rule = EpochRule::new(2);

    // Act
    let held = [0, 2, 3].map(|epoch| rule.held::<u64>(&membership(epoch)));

    // Assert
    assert_eq!(
        held,
        [
            Some(Outcome::MembershipIgnored { epoch: 0 }),
            Some(Outcome::MembershipIgnored { epoch: 2 }),
            None,
        ]
    );
}

#[test]
fn refused_of_a_claim_below_the_epoch_sends_the_membership() {
    // Arrange
    let rule = EpochRule::new(2);

    // Act
    let refused = [rule.refused::<u64>(3, 1), rule.refused::<u64>(3, 2)];

    // Assert
    assert_eq!(
        refused,
        [
            Some(vec![
                Outcome::StaleEpochIgnored {
                    peer_id: 3,
                    epoch: 1
                },
                Outcome::SendMembership { peer_id: 3 },
            ]),
            None,
        ]
    );
}

#[test]
fn signal_refused_of_a_signal_from_behind_is_refused_and_of_anything_else_none() {
    // Arrange
    let rule = EpochRule::new(1);

    // Act
    let refused = [
        rule.signal_refused::<u64>(&Command::ReadyObserved {
            peer_id: 2,
            epoch: 0,
        })
        .is_some(),
        rule.signal_refused::<u64>(&Command::ParticipationObserved {
            peer_id: 2,
            epoch: 0,
        })
        .is_some(),
        rule.signal_refused::<u64>(&Command::MembershipAcknowledged {
            peer_id: 2,
            epoch: 0,
        })
        .is_some(),
    ];

    // Assert
    assert_eq!(refused, [true, true, false]);
}

#[test]
fn uncounted_of_a_non_member_s_signal_ignores_it_and_of_a_member_s_from_behind_refuses_it() {
    // Arrange
    let rule = EpochRule::new(1);
    let members = Members::new(vec![0, 1, 2]);

    // Act
    let uncounted = [
        rule.uncounted::<u64>(
            &Command::ParticipationObserved {
                peer_id: 9,
                epoch: 1,
            },
            &members,
        ),
        rule.uncounted::<u64>(
            &Command::ReadyObserved {
                peer_id: 2,
                epoch: 0,
            },
            &members,
        ),
        rule.uncounted::<u64>(
            &Command::ReadyObserved {
                peer_id: 2,
                epoch: 1,
            },
            &members,
        ),
    ];

    // Assert
    assert_eq!(
        uncounted,
        [
            Some(vec![Outcome::NonMemberIgnored { peer_id: 9 }]),
            Some(vec![
                Outcome::StaleEpochIgnored {
                    peer_id: 2,
                    epoch: 0
                },
                Outcome::SendMembership { peer_id: 2 },
            ]),
            None,
        ]
    );
}
