// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use faction::outcome::Outcome;
use faction::states::compute_output::ObservedKind;
use faction::states::compute_output::ObservedOutput;
use faction::PeerId;

const PEER_ID: PeerId = 42;

#[test]
fn observed_kind_debug_and_clone_and_eq() {
    // Arrange & Act & Assert
    assert_eq!(ObservedKind::Participation, ObservedKind::Participation);
    assert_eq!(ObservedKind::Ready, ObservedKind::Ready);
    assert_ne!(ObservedKind::Participation, ObservedKind::Ready);
    let _ = format!("{:?}", ObservedKind::Participation);
    let _ = format!("{:?}", ObservedKind::Ready);
    let cloned = ObservedKind::Participation;
    assert_eq!(cloned, ObservedKind::Participation);
}

#[test]
fn new_creates_observed_output_with_participation_kind() {
    // Arrange & Act
    let output = ObservedOutput::new(ObservedKind::Participation, PEER_ID, false);

    // Assert
    assert_eq!(
        output.outcome().clone(),
        Outcome::ParticipationAccepted { peer_id: PEER_ID }
    );
}

#[test]
fn new_creates_observed_output_with_ready_kind() {
    // Arrange & Act
    let output = ObservedOutput::new(ObservedKind::Ready, PEER_ID, false);

    // Assert
    assert_eq!(
        output.outcome().clone(),
        Outcome::ReadyAccepted { peer_id: PEER_ID }
    );
}

#[test]
fn compute_output_participation_duplicate_returns_duplicate_participation_ignored() {
    // Arrange & Act
    let output = ObservedOutput::new(ObservedKind::Participation, PEER_ID, true);

    // Assert
    assert_eq!(
        output.outcome().clone(),
        Outcome::DuplicateParticipationIgnored { peer_id: PEER_ID }
    );
}

#[test]
fn compute_output_ready_duplicate_returns_duplicate_ready_ignored() {
    // Arrange & Act
    let output = ObservedOutput::new(ObservedKind::Ready, PEER_ID, true);

    // Assert
    assert_eq!(
        output.outcome().clone(),
        Outcome::DuplicateReadyIgnored { peer_id: PEER_ID }
    );
}

#[test]
fn compute_output_participation_timely_returns_participation_accepted() {
    // Arrange & Act
    let output = ObservedOutput::new(ObservedKind::Participation, PEER_ID, false);

    // Assert
    assert_eq!(
        output.outcome().clone(),
        Outcome::ParticipationAccepted { peer_id: PEER_ID }
    );
}

#[test]
fn compute_output_ready_timely_returns_ready_accepted() {
    // Arrange & Act
    let output = ObservedOutput::new(ObservedKind::Ready, PEER_ID, false);

    // Assert
    assert_eq!(
        output.outcome().clone(),
        Outcome::ReadyAccepted { peer_id: PEER_ID }
    );
}

#[test]
fn compute_output_preserves_peer_id_in_output() {
    // Arrange
    let peer_id: PeerId = 99;

    // Act
    let output = ObservedOutput::new(ObservedKind::Participation, peer_id, false);

    // Assert
    assert_eq!(
        output.outcome().clone(),
        Outcome::ParticipationAccepted { peer_id: 99 }
    );
}
