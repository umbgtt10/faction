// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::conclusion::Conclusion;
use faction::outcome::Outcome;
use faction::states::observed_step::ObservedKind;
use faction::states::observed_step::ObservedStep;

#[test]
fn new_adds_peer_when_not_dup() {
    // Arrange
    let confirmed = vec![1, 2];

    // Act
    let step = ObservedStep::new(confirmed, 3, ObservedKind::Participation, None);

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
    let step = ObservedStep::new(confirmed.clone(), 2, ObservedKind::Ready, None);

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
    let step = ObservedStep::new(vec![], 0, ObservedKind::Participation, None);

    // Act & Assert
    assert!(!step.is_quorum());
    assert_eq!(step.outcomes().len(), 1);
}

#[test]
fn new_local_adds_peer_when_not_present() {
    // Arrange
    let confirmed = vec![1, 2];

    // Act
    let step = ObservedStep::new_local(confirmed, 3, 5);

    // Assert
    assert_eq!(step.confirmed_peers(), vec![1, 2, 3]);
    assert!(!step.is_quorum());
    assert_eq!(
        step.outcomes(),
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
}

#[test]
fn new_local_always_confirmed_new_even_when_already_present() {
    // Arrange
    let confirmed = vec![1, 2, 3];

    // Act
    let step = ObservedStep::new_local(confirmed.clone(), 2, 5);

    // Assert
    assert_eq!(step.confirmed_peers(), confirmed);
    assert!(!step.is_quorum());
}

#[test]
fn new_local_reaches_quorum_when_count_meets_threshold() {
    // Arrange
    let confirmed = vec![1, 2, 3];

    // Act
    let step = ObservedStep::new_local(confirmed, 99, 3);

    // Assert
    assert!(step.is_quorum());
    assert_eq!(
        step.outcomes(),
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
}

#[test]
fn new_local_does_not_reach_quorum_below_threshold() {
    // Arrange
    let confirmed = vec![1];

    // Act
    let step = ObservedStep::new_local(confirmed, 2, 5);

    // Assert
    assert!(!step.is_quorum());
    assert_eq!(step.outcomes().len(), 2);
}
