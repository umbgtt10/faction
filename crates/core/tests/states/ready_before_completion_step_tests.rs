// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::outcome::Outcome;
use faction::states::ready_before_completion_step::ReadyBeforeCompletionStep;

extern crate alloc;

#[test]
fn new_of_a_peer_already_ready_ignores_the_repeat() {
    // Arrange
    let collecting = vec![1, 2];

    // Act
    let step = ReadyBeforeCompletionStep::new(collecting.clone(), 2);

    // Assert
    assert_eq!(step.collecting_peers(), collecting);
    assert_eq!(
        step.outcomes(),
        vec![Outcome::DuplicateReadyIgnored { peer_id: 2 }]
    );
}

#[test]
fn new_of_a_peer_not_yet_ready_accepts_it_as_the_last_collecting_peer() {
    // Arrange
    let collecting = vec![1, 2];

    // Act
    let step = ReadyBeforeCompletionStep::new(collecting, 3);

    // Assert
    assert_eq!(step.collecting_peers(), vec![1, 2, 3]);
    assert_eq!(step.outcomes(), vec![Outcome::ReadyAccepted { peer_id: 3 }]);
}
