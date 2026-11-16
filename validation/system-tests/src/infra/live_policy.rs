// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::sync::Mutex;

// A fault policy shared between whoever changes it and the fault layer that
// reads it: a clone holds the same policy, so one set while a cluster runs
// takes at the layer's next call.
#[derive(Debug, Clone, Default)]
pub struct LivePolicy<P> {
    policy: Arc<Mutex<P>>,
}

impl<P: Clone> LivePolicy<P> {
    #[must_use]
    pub fn new(policy: P) -> Self {
        Self {
            policy: Arc::new(Mutex::new(policy)),
        }
    }

    #[must_use]
    pub fn current(&self) -> P {
        self.policy.lock().unwrap().clone()
    }

    pub fn set(&self, policy: P) {
        *self.policy.lock().unwrap() = policy;
    }
}
