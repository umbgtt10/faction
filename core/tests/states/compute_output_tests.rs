// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use faction::freshness_classification::FreshnessClassification;
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
    let output = ObservedOutput::new(ObservedKind::Participation, PEER_ID);

    // Assert
    let result = output.compute_output(FreshnessClassification::Timely, false);
    assert_eq!(result, Outcome::ParticipationAccepted { peer_id: PEER_ID });
}

#[test]
fn new_creates_observed_output_with_ready_kind() {
    // Arrange & Act
    let output = ObservedOutput::new(ObservedKind::Ready, PEER_ID);

    // Assert
    let result = output.compute_output(FreshnessClassification::Timely, false);
    assert_eq!(result, Outcome::ReadyAccepted { peer_id: PEER_ID });
}

#[test]
fn compute_output_participation_stale_returns_stale_participation_ignored() {
    // Arrange
    let output = ObservedOutput::new(ObservedKind::Participation, PEER_ID);

    // Act
    let result = output.compute_output(FreshnessClassification::Stale, false);

    // Assert
    assert_eq!(
        result,
        Outcome::StaleParticipationIgnored { peer_id: PEER_ID }
    );
}

#[test]
fn compute_output_ready_stale_returns_stale_ready_ignored() {
    // Arrange
    let output = ObservedOutput::new(ObservedKind::Ready, PEER_ID);

    // Act
    let result = output.compute_output(FreshnessClassification::Stale, false);

    // Assert
    assert_eq!(result, Outcome::StaleReadyIgnored { peer_id: PEER_ID });
}

#[test]
fn compute_output_stale_dominates_over_duplicate() {
    // Arrange
    let output = ObservedOutput::new(ObservedKind::Participation, PEER_ID);

    // Act
    let result = output.compute_output(FreshnessClassification::Stale, true);

    // Assert
    assert_eq!(
        result,
        Outcome::StaleParticipationIgnored { peer_id: PEER_ID }
    );
}

#[test]
fn compute_output_participation_duplicate_returns_duplicate_participation_ignored() {
    // Arrange
    let output = ObservedOutput::new(ObservedKind::Participation, PEER_ID);

    // Act
    let result = output.compute_output(FreshnessClassification::Timely, true);

    // Assert
    assert_eq!(
        result,
        Outcome::DuplicateParticipationIgnored { peer_id: PEER_ID }
    );
}

#[test]
fn compute_output_ready_duplicate_returns_duplicate_ready_ignored() {
    // Arrange
    let output = ObservedOutput::new(ObservedKind::Ready, PEER_ID);

    // Act
    let result = output.compute_output(FreshnessClassification::Timely, true);

    // Assert
    assert_eq!(result, Outcome::DuplicateReadyIgnored { peer_id: PEER_ID });
}

#[test]
fn compute_output_duplicate_with_delayed_classification_still_returns_duplicate() {
    // Arrange
    let output = ObservedOutput::new(ObservedKind::Ready, PEER_ID);

    // Act
    let result = output.compute_output(FreshnessClassification::DelayedWithinMargin, true);

    // Assert
    assert_eq!(result, Outcome::DuplicateReadyIgnored { peer_id: PEER_ID });
}

#[test]
fn compute_output_participation_timely_returns_participation_accepted() {
    // Arrange
    let output = ObservedOutput::new(ObservedKind::Participation, PEER_ID);

    // Act
    let result = output.compute_output(FreshnessClassification::Timely, false);

    // Assert
    assert_eq!(result, Outcome::ParticipationAccepted { peer_id: PEER_ID });
}

#[test]
fn compute_output_ready_timely_returns_ready_accepted() {
    // Arrange
    let output = ObservedOutput::new(ObservedKind::Ready, PEER_ID);

    // Act
    let result = output.compute_output(FreshnessClassification::Timely, false);

    // Assert
    assert_eq!(result, Outcome::ReadyAccepted { peer_id: PEER_ID });
}

#[test]
fn compute_output_participation_delayed_returns_delayed_participation_accepted() {
    // Arrange
    let output = ObservedOutput::new(ObservedKind::Participation, PEER_ID);

    // Act
    let result = output.compute_output(FreshnessClassification::DelayedWithinMargin, false);

    // Assert
    assert_eq!(
        result,
        Outcome::DelayedParticipationAccepted { peer_id: PEER_ID }
    );
}

#[test]
fn compute_output_ready_delayed_returns_delayed_ready_accepted() {
    // Arrange
    let output = ObservedOutput::new(ObservedKind::Ready, PEER_ID);

    // Act
    let result = output.compute_output(FreshnessClassification::DelayedWithinMargin, false);

    // Assert
    assert_eq!(result, Outcome::DelayedReadyAccepted { peer_id: PEER_ID });
}

#[test]
fn compute_output_preserves_peer_id_in_output() {
    // Arrange
    let peer_id: PeerId = 99;
    let output = ObservedOutput::new(ObservedKind::Participation, peer_id);

    // Act
    let result = output.compute_output(FreshnessClassification::Timely, false);

    // Assert
    assert_eq!(result, Outcome::ParticipationAccepted { peer_id: 99 });
}
