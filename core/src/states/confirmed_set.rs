// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

use crate::freshness_classification::FreshnessClassification;
use crate::PeerId;

use super::bitmap::Bitmap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmedSet {
    bits: Bitmap,
}

impl ConfirmedSet {
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            bits: Bitmap::new(size),
        }
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.bits.count()
    }

    #[must_use]
    pub fn is_confirmed(&self, index: usize) -> bool {
        self.bits.is_set(index)
    }

    #[must_use]
    pub fn try_confirm(
        &self,
        index: Option<usize>,
        is_dup: bool,
        classification: Option<FreshnessClassification>,
    ) -> (Self, bool) {
        match (index, is_dup, classification) {
            (Some(i), false, Some(c))
                if c != FreshnessClassification::Stale && i < self.bits.len() =>
            {
                let (bits, confirmed) = self.bits.set(i);
                (Self { bits }, confirmed)
            }
            _ => (self.clone(), false),
        }
    }

    #[must_use]
    pub fn confirm(&self, index: usize) -> (Self, bool) {
        let (bits, confirmed) = self.bits.set(index);
        (Self { bits }, confirmed)
    }

    #[must_use]
    pub fn confirmed_peers(&self, peer_set: &[PeerId]) -> Vec<PeerId> {
        self.bits.peer_ids(peer_set)
    }
}
