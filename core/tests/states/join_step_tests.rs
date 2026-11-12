// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::command::Command;
use faction::members::Members;
use faction::outcome::Outcome;
use faction::states::join_step::JoinStep;

extern crate alloc;

fn members() -> Members {
    Members::new(vec![0, 1, 2])
}

#[test]
fn new_join_approved_member_returns_duplicate_ignored() {
    // Arrange & Act
    let join = JoinStep::new(members(), &Command::JoinApproved { peer_id: 1 });

    // Assert
    assert_eq!(
        join.outcomes(),
        &[Outcome::DuplicateMemberIgnored { peer_id: 1 }]
    );
    assert_eq!(join.members(), &members());
}

#[test]
fn new_join_approved_non_member_returns_member_admitted() {
    // Arrange & Act
    let join = JoinStep::new(members(), &Command::JoinApproved { peer_id: 5 });

    // Assert
    assert_eq!(join.outcomes(), &[Outcome::MemberAdmitted { peer_id: 5 }]);
    assert_eq!(join.members(), &Members::new(vec![0, 1, 2, 5]));
}

#[test]
fn new_join_rejected_returns_join_denied() {
    // Arrange & Act
    let join = JoinStep::new(members(), &Command::JoinRejected { peer_id: 5 });

    // Assert
    assert_eq!(join.outcomes(), &[Outcome::JoinDenied { peer_id: 5 }]);
    assert_eq!(join.members(), &members());
}

#[test]
fn new_join_requested_returns_emit_join_request() {
    // Arrange & Act
    let join = JoinStep::new(members(), &Command::JoinRequested { peer_id: 5 });

    // Assert
    assert_eq!(join.outcomes(), &[Outcome::EmitJoinRequest { peer_id: 5 }]);
    assert_eq!(join.members(), &members());
}

#[test]
#[should_panic(expected = "JoinStep only handles join commands")]
fn new_non_join_command_panics() {
    // Arrange
    let members = members();

    // Act & Assert
    let _ = JoinStep::new(members, &Command::DeadlineExpired);
}
