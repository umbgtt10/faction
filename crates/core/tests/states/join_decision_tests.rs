// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction::states::join_decision::JoinDecision;

#[test]
fn of_a_signal_returns_none() {
    // Arrange & Act
    let decision = JoinDecision::of(&Command::ParticipationObserved { peer_id: 6 });

    // Assert
    assert_eq!(decision, None);
}

#[test]
fn of_join_approved_returns_approved() {
    // Arrange & Act
    let decision = JoinDecision::of(&Command::JoinApproved { peer_id: 4 });

    // Assert
    assert_eq!(decision, Some(JoinDecision::Approved(4)));
}

#[test]
fn of_join_rejected_returns_rejected() {
    // Arrange & Act
    let decision = JoinDecision::of(&Command::JoinRejected { peer_id: 5 });

    // Assert
    assert_eq!(decision, Some(JoinDecision::Rejected(5)));
}

#[test]
fn of_join_requested_returns_requested() {
    // Arrange & Act
    let decision = JoinDecision::of(&Command::JoinRequested { peer_id: 6 });

    // Assert
    assert_eq!(decision, Some(JoinDecision::Requested(6)));
}
