// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

use crate::freshness_classification::FreshnessClassification;
use crate::PeerId;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfirmedSet {
    confirmed: Vec<PeerId>,
}

impl ConfirmedSet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.confirmed.len()
    }

    #[must_use]
    pub fn is_confirmed(&self, peer_id: PeerId) -> bool {
        self.confirmed.contains(&peer_id)
    }

    #[must_use]
    pub fn confirmed_peers(&self) -> &[PeerId] {
        &self.confirmed
    }

    #[must_use]
    pub fn confirm(&self, peer_id: PeerId) -> (Self, bool) {
        if self.confirmed.contains(&peer_id) {
            return (self.clone(), false);
        }
        let mut confirmed = self.confirmed.clone();
        confirmed.push(peer_id);
        (Self { confirmed }, true)
    }

    #[must_use]
    pub fn try_confirm(
        &self,
        peer_id: PeerId,
        classification: FreshnessClassification,
    ) -> (Self, bool) {
        if matches!(classification, FreshnessClassification::Stale) {
            return (self.clone(), false);
        }
        if self.confirmed.contains(&peer_id) {
            return (self.clone(), false);
        }
        let mut confirmed = self.confirmed.clone();
        confirmed.push(peer_id);
        (Self { confirmed }, true)
    }
}
