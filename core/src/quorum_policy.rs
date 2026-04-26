// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
