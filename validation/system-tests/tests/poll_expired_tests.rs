// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::poll_expired::PollExpired;
use std::time::Duration;

#[test]
fn to_string_names_what_was_awaited_and_for_how_long() {
    // Arrange
    let expired = PollExpired {
        waiting_for: String::from("every node bootstrapped"),
        deadline: Duration::from_millis(250),
    };

    // Act
    let text = expired.to_string();

    // Assert
    assert_eq!(
        text,
        "gave up after 250ms waiting for every node bootstrapped"
    );
}
