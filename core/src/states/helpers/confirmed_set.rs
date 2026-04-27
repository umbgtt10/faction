// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

use crate::freshness_classification::FreshnessClassification;

#[derive(Debug, Clone)]
pub struct ConfirmedSet {
    flags: Vec<bool>,
    count: usize,
}

impl ConfirmedSet {
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            flags: alloc::vec![false; size],
            count: 0,
        }
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }

    #[must_use]
    pub fn is_confirmed(&self, index: usize) -> bool {
        self.flags[index]
    }

    #[must_use]
    pub fn try_confirm(
        &self,
        index: Option<usize>,
        is_dup: bool,
        classification: Option<FreshnessClassification>,
    ) -> (Self, bool) {
        match (index, is_dup, classification) {
            (Some(i), false, Some(c)) if c != FreshnessClassification::Stale => {
                let mut new_flags = self.flags.clone();
                new_flags[i] = true;
                (
                    Self {
                        flags: new_flags,
                        count: self.count + 1,
                    },
                    true,
                )
            }
            _ => (self.clone(), false),
        }
    }

    #[must_use]
    pub fn confirm(&self, index: usize) -> (Self, bool) {
        if self.flags[index] {
            (self.clone(), false)
        } else {
            let mut new_flags = self.flags.clone();
            new_flags[index] = true;
            (
                Self {
                    flags: new_flags,
                    count: self.count + 1,
                },
                true,
            )
        }
    }
}
