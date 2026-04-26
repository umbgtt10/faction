// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::freshness_classification::FreshnessClassification;
use crate::Freshness;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreshnessPolicy {
    max_delay: Freshness,
}

impl FreshnessPolicy {
    #[must_use]
    pub const fn new(max_delay: Freshness) -> Self {
        Self { max_delay }
    }

    #[must_use]
    pub const fn max_delay(&self) -> Freshness {
        self.max_delay
    }

    #[must_use]
    pub const fn classify(
        &self,
        current_marker: Freshness,
        observed_marker: Freshness,
    ) -> FreshnessClassification {
        if observed_marker > current_marker {
            return FreshnessClassification::Stale;
        }

        let age = current_marker - observed_marker;
        if age == 0 {
            FreshnessClassification::Timely
        } else if age <= self.max_delay {
            FreshnessClassification::DelayedWithinMargin
        } else {
            FreshnessClassification::Stale
        }
    }
}
