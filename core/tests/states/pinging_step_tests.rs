// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::vec;

use faction::outcome::Outcome;
use faction::states::pinging_step::PingingStep;

#[test]
fn new_adds_peer_when_not_dup() {
    // Arrange
    let confirmed = vec![1, 2];

    // Act
    let step = PingingStep::new(confirmed, 3);

    // Assert
    assert_eq!(step.confirmed_peers(), vec![1, 2, 3]);
    assert_eq!(
        step.outcomes(),
        vec![Outcome::ParticipationAccepted { peer_id: 3 }]
    );
}

#[test]
fn new_does_not_add_when_duplicate() {
    // Arrange
    let confirmed = vec![1, 2];

    // Act
    let step = PingingStep::new(confirmed.clone(), 2);

    // Assert
    assert_eq!(step.confirmed_peers(), confirmed);
    assert_eq!(
        step.outcomes(),
        vec![Outcome::DuplicateParticipationIgnored { peer_id: 2 }]
    );
}
