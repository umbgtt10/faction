// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec;
use alloc::vec::Vec;

use crate::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitmap {
    bits: Vec<bool>,
    count: usize,
}

impl Bitmap {
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            bits: vec![false; size],
            count: 0,
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }

    #[must_use]
    pub fn is_set(&self, index: usize) -> bool {
        self.bits.get(index).copied().unwrap_or(false)
    }

    #[must_use]
    pub fn set(&self, index: usize) -> (Self, bool) {
        if index >= self.bits.len() || self.bits[index] {
            return (self.clone(), false);
        }

        let mut bits = self.bits.clone();
        bits[index] = true;
        (
            Self {
                bits,
                count: self.count + 1,
            },
            true,
        )
    }

    pub fn set_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.bits
            .iter()
            .enumerate()
            .filter(|(_, &set)| set)
            .map(|(i, _)| i)
    }

    #[must_use]
    pub fn peer_ids(&self, peer_set: &[PeerId]) -> Vec<PeerId> {
        self.bits
            .iter()
            .enumerate()
            .filter(|(_, &set)| set)
            .map(|(i, _)| peer_set[i])
            .collect()
    }
}
