// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::conclusion::Conclusion;
use faction::outcome::Outcome;
use faction::states::collecting_step::CollectingStep;

#[test]
fn new_adds_peer_when_not_dup() {
    // Arrange
    let confirmed = vec![1, 2];

    // Act
    let step = CollectingStep::new(confirmed, 3, None);

    // Assert
    assert_eq!(step.confirmed_peers(), vec![1, 2, 3]);
    assert_eq!(step.outcomes(), vec![Outcome::ReadyAccepted { peer_id: 3 }]);
}

#[test]
fn new_does_not_add_when_duplicate() {
    // Arrange
    let confirmed = vec![1, 2];

    // Act
    let step = CollectingStep::new(confirmed.clone(), 2, None);

    // Assert
    assert_eq!(step.confirmed_peers(), confirmed);
    assert_eq!(
        step.outcomes(),
        vec![Outcome::DuplicateReadyIgnored { peer_id: 2 }]
    );
}

#[test]
fn new_with_none_threshold_never_quorum() {
    // Arrange
    let step = CollectingStep::new(vec![], 0, None);

    // Act & Assert
    assert!(!step.is_quorum());
    assert_eq!(step.outcomes().len(), 1);
}

#[test]
fn new_reaches_quorum_when_threshold_is_met() {
    // Arrange
    let step = CollectingStep::new(vec![1, 2, 3], 99, Some(4));

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
