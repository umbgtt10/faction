// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use faction::freshness_classification::FreshnessClassification;
use faction::states::confirmed_set::ConfirmedSet;

#[test]
fn new_creates_empty_set() {
    // Arrange & Act
    let set = ConfirmedSet::new();

    // Assert
    assert_eq!(set.count(), 0);
}

#[test]
fn is_confirmed_returns_false_for_unconfirmed_peer() {
    // Arrange
    let set = ConfirmedSet::new();

    // Act & Assert
    for peer_id in 0..5 {
        assert!(!set.is_confirmed(peer_id));
    }
}

#[test]
fn is_confirmed_returns_true_after_confirm() {
    // Arrange
    let set = ConfirmedSet::new();
    let (set, _) = set.confirm(2);

    // Act & Assert
    assert!(set.is_confirmed(2));
}

#[test]
fn try_confirm_stale_returns_no_change() {
    // Arrange
    let set = ConfirmedSet::new();

    // Act
    let (set, was_newly_confirmed) = set.try_confirm(0, FreshnessClassification::Stale);

    // Assert
    assert!(!was_newly_confirmed);
    assert_eq!(set.count(), 0);
}

#[test]
fn try_confirm_already_confirmed_returns_no_change() {
    // Arrange
    let set = ConfirmedSet::new();
    let (set, _) = set.try_confirm(0, FreshnessClassification::Timely);
    assert_eq!(set.count(), 1);

    // Act
    let (set, was_newly_confirmed) = set.try_confirm(0, FreshnessClassification::Timely);

    // Assert
    assert!(!was_newly_confirmed);
    assert_eq!(set.count(), 1);
}

#[test]
fn try_confirm_timely_confirms() {
    // Arrange
    let set = ConfirmedSet::new();

    // Act
    let (set, was_newly_confirmed) = set.try_confirm(2, FreshnessClassification::Timely);

    // Assert
    assert!(was_newly_confirmed);
    assert!(set.is_confirmed(2));
    assert_eq!(set.count(), 1);
}

#[test]
fn try_confirm_delayed_confirms() {
    // Arrange
    let set = ConfirmedSet::new();

    // Act
    let (set, was_newly_confirmed) =
        set.try_confirm(3, FreshnessClassification::DelayedWithinMargin);

    // Assert
    assert!(was_newly_confirmed);
    assert!(set.is_confirmed(3));
    assert_eq!(set.count(), 1);
}

#[test]
fn try_confirm_multiple_distinct_peers_increment_count() {
    // Arrange
    let set = ConfirmedSet::new();

    // Act
    let (set, _) = set.try_confirm(10, FreshnessClassification::Timely);
    let (set, _) = set.try_confirm(20, FreshnessClassification::Timely);
    let (set, _) = set.try_confirm(30, FreshnessClassification::Timely);

    // Assert
    assert_eq!(set.count(), 3);
    assert!(set.is_confirmed(10));
    assert!(set.is_confirmed(20));
    assert!(set.is_confirmed(30));
    assert!(!set.is_confirmed(99));
}

#[test]
fn try_confirm_does_not_mutate_original() {
    // Arrange
    let set = ConfirmedSet::new();

    // Act
    let (new_set, was_newly_confirmed) = set.try_confirm(0, FreshnessClassification::Timely);

    // Assert
    assert!(was_newly_confirmed);
    assert!(new_set.is_confirmed(0));
    assert_eq!(new_set.count(), 1);
    assert_eq!(set.count(), 0);
    assert!(!set.is_confirmed(0));
}

#[test]
fn confirm_new_peer_confirms() {
    // Arrange
    let set = ConfirmedSet::new();

    // Act
    let (set, was_newly_confirmed) = set.confirm(1);

    // Assert
    assert!(was_newly_confirmed);
    assert!(set.is_confirmed(1));
    assert_eq!(set.count(), 1);
}

#[test]
fn confirm_existing_peer_returns_no_change() {
    // Arrange
    let set = ConfirmedSet::new();
    let (set, _) = set.confirm(1);
    assert_eq!(set.count(), 1);

    // Act
    let (set, was_newly_confirmed) = set.confirm(1);

    // Assert
    assert!(!was_newly_confirmed);
    assert!(set.is_confirmed(1));
    assert_eq!(set.count(), 1);
}

#[test]
fn confirm_does_not_mutate_original() {
    // Arrange
    let set = ConfirmedSet::new();

    // Act
    let (new_set, was_newly_confirmed) = set.confirm(1);

    // Assert
    assert!(was_newly_confirmed);
    assert!(new_set.is_confirmed(1));
    assert_eq!(new_set.count(), 1);
    assert_eq!(set.count(), 0);
    assert!(!set.is_confirmed(1));
}

#[test]
fn confirm_then_try_confirm_duplicate_returns_no_change() {
    // Arrange
    let set = ConfirmedSet::new();
    let (set, _) = set.confirm(2);
    assert_eq!(set.count(), 1);

    // Act
    let (set, was_newly_confirmed) = set.try_confirm(2, FreshnessClassification::Timely);

    // Assert
    assert!(!was_newly_confirmed);
    assert_eq!(set.count(), 1);
}

#[test]
fn try_confirm_then_confirm_same_peer_returns_no_change() {
    // Arrange
    let set = ConfirmedSet::new();
    let (set, _) = set.try_confirm(0, FreshnessClassification::Timely);
    assert_eq!(set.count(), 1);

    // Act
    let (set, was_newly_confirmed) = set.confirm(0);

    // Assert
    assert!(!was_newly_confirmed);
    assert_eq!(set.count(), 1);
}

#[test]
fn cloned_set_preserves_count_and_peers() {
    // Arrange
    let set = ConfirmedSet::new();
    let (set, _) = set.confirm(0);
    let (set, _) = set.confirm(2);
    let (set, _) = set.confirm(4);

    // Act
    let cloned = set.clone();

    // Assert
    assert_eq!(cloned.count(), 3);
    assert!(cloned.is_confirmed(0));
    assert!(cloned.is_confirmed(2));
    assert!(cloned.is_confirmed(4));
    assert!(!cloned.is_confirmed(1));
    assert!(!cloned.is_confirmed(3));
}

#[test]
fn debug_format_does_not_panic() {
    // Arrange
    let set = ConfirmedSet::new();

    // Act & Assert
    let _ = format!("{:?}", set);
}
