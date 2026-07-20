// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuorumPolicy {
    threshold: usize,
}

impl QuorumPolicy {
    #[must_use]
    pub const fn new(threshold: usize) -> Self {
        Self { threshold }
    }

    #[must_use]
    pub const fn threshold(&self) -> usize {
        self.threshold
    }

    #[must_use]
    pub const fn is_satisfied(&self, confirmed_count: usize) -> bool {
        confirmed_count >= self.threshold
    }
}
