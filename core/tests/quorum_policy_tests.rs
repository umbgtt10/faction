// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use faction::quorum_policy::QuorumPolicy;

#[test]
fn new_stores_threshold() {
    // Arrange & Act
    let policy = QuorumPolicy::new(5);

    // Assert
    assert_eq!(policy.threshold(), 5);
}

#[test]
fn threshold_returns_configured_value() {
    // Arrange & Act
    let policy = QuorumPolicy::new(3);

    // Assert
    assert_eq!(policy.threshold(), 3);
}

#[test]
fn is_satisfied_returns_true_when_met_exactly() {
    // Arrange
    let policy = QuorumPolicy::new(4);

    // Act & Assert
    assert!(policy.is_satisfied(4));
}

#[test]
fn is_satisfied_returns_true_when_exceeded() {
    // Arrange
    let policy = QuorumPolicy::new(4);

    // Act & Assert
    assert!(policy.is_satisfied(5));
    assert!(policy.is_satisfied(10));
}

#[test]
fn is_satisfied_returns_false_when_below_threshold() {
    // Arrange
    let policy = QuorumPolicy::new(4);

    // Act & Assert
    assert!(!policy.is_satisfied(3));
    assert!(!policy.is_satisfied(0));
}

#[test]
fn is_satisfied_zero_threshold() {
    // Arrange
    let policy = QuorumPolicy::new(0);

    // Act & Assert
    assert!(policy.is_satisfied(0));
    assert!(policy.is_satisfied(1));
}
