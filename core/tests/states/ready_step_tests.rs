// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::vec;

use faction::conclusion::Conclusion;
use faction::outcome::Outcome;
use faction::states::ready_step::ReadyStep;

#[test]
fn new_adds_peer_when_not_dup() {
    // Arrange
    let confirmed = vec![1, 2];

    // Act
    let step = ReadyStep::new(confirmed, 3, 10);

    // Assert
    assert_eq!(step.confirmed_peers(), vec![1, 2, 3]);
    assert!(!step.is_quorum());
    assert_eq!(step.outcomes(), vec![Outcome::ReadyAccepted { peer_id: 3 }]);
}

#[test]
fn new_does_not_add_when_duplicate() {
    // Arrange
    let confirmed = vec![1, 2];

    // Act
    let step = ReadyStep::new(confirmed.clone(), 2, 10);

    // Assert
    assert_eq!(step.confirmed_peers(), confirmed);
    assert!(!step.is_quorum());
    assert_eq!(
        step.outcomes(),
        vec![Outcome::DuplicateReadyIgnored { peer_id: 2 }]
    );
}

#[test]
fn new_reaches_quorum_when_threshold_is_met() {
    // Arrange
    let step = ReadyStep::new(vec![1, 2, 3], 99, 4);

    // Act & Assert
    assert!(step.is_quorum());
    assert_eq!(
        step.outcomes(),
        vec![
            Outcome::ReadyAccepted { peer_id: 99 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
}
