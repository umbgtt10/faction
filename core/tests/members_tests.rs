// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::command::Command;
use faction::members::Members;

extern crate alloc;

fn members() -> Members {
    Members::new(vec![0, 1, 2])
}

#[test]
fn is_empty_no_peers_returns_true() {
    // Arrange
    let members = Members::new(vec![]);

    // Act
    let is_empty = members.is_empty();

    // Assert
    assert!(is_empty);
}

#[test]
fn is_empty_with_peers_returns_false() {
    // Arrange
    let members = members();

    // Act
    let is_empty = members.is_empty();

    // Assert
    assert!(!is_empty);
}

#[test]
fn non_member_observed_join_request_from_non_member_returns_none() {
    // Arrange
    let members = members();

    // Act
    let non_member = members.non_member_observed(&Command::JoinRequested { peer_id: 9 });

    // Assert
    assert_eq!(non_member, None);
}

#[test]
fn non_member_observed_participation_from_member_returns_none() {
    // Arrange
    let members = members();

    // Act
    let non_member = members.non_member_observed(&Command::ParticipationObserved { peer_id: 1 });

    // Assert
    assert_eq!(non_member, None);
}

#[test]
fn non_member_observed_participation_from_non_member_returns_peer() {
    // Arrange
    let members = members();

    // Act
    let non_member = members.non_member_observed(&Command::ParticipationObserved { peer_id: 9 });

    // Assert
    assert_eq!(non_member, Some(9));
}

#[test]
fn non_member_observed_ready_from_member_returns_none() {
    // Arrange
    let members = members();

    // Act
    let non_member = members.non_member_observed(&Command::ReadyObserved { peer_id: 2 });

    // Assert
    assert_eq!(non_member, None);
}

#[test]
fn non_member_observed_ready_from_non_member_returns_peer() {
    // Arrange
    let members = members();

    // Act
    let non_member = members.non_member_observed(&Command::ReadyObserved { peer_id: 7 });

    // Assert
    assert_eq!(non_member, Some(7));
}

#[test]
fn with_admitted_member_returns_same_members() {
    // Arrange
    let members = members();

    // Act
    let admitted = members.with_admitted(1);

    // Assert
    assert_eq!(admitted, Members::new(vec![0, 1, 2]));
}

#[test]
fn with_admitted_non_member_appends_peer() {
    // Arrange
    let members = members();

    // Act
    let admitted = members.with_admitted(5);

    // Assert
    assert_eq!(admitted, Members::new(vec![0, 1, 2, 5]));
}
