// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::freshness_classification::FreshnessClassification;
use faction::freshness_policy::FreshnessPolicy;

#[test]
fn classify_returns_timely_when_observation_matches_current_marker() {
    // Arrange & Act
    let policy = FreshnessPolicy::new(2);
    let classification = policy.classify(10, 10);

    // Assert
    assert_eq!(classification, FreshnessClassification::Timely);
}

#[test]
fn classify_returns_delayed_within_margin_when_age_is_inside_margin() {
    // Arrange & Act
    let policy = FreshnessPolicy::new(2);
    let classification = policy.classify(10, 9);

    // Assert
    assert_eq!(classification, FreshnessClassification::DelayedWithinMargin);
}

#[test]
fn classify_returns_delayed_within_margin_when_age_matches_margin() {
    // Arrange & Act
    let policy = FreshnessPolicy::new(2);
    let classification = policy.classify(10, 8);

    // Assert
    assert_eq!(classification, FreshnessClassification::DelayedWithinMargin);
}

#[test]
fn classify_returns_stale_when_age_exceeds_margin() {
    // Arrange & Act
    let policy = FreshnessPolicy::new(2);
    let classification = policy.classify(10, 7);

    // Assert
    assert_eq!(classification, FreshnessClassification::Stale);
}

#[test]
fn classify_returns_stale_when_observation_marker_is_in_the_future() {
    // Arrange & Act
    let policy = FreshnessPolicy::new(2);
    let classification = policy.classify(10, 11);

    // Assert
    assert_eq!(classification, FreshnessClassification::Stale);
}

#[test]
fn max_delay_returns_configured_value() {
    // Arrange & Act
    let policy = FreshnessPolicy::new(3);
    let max_delay = policy.max_delay();

    // Assert
    assert_eq!(max_delay, 3);
}
