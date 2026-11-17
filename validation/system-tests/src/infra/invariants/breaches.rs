// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::sync::Mutex;

use crate::infra::invariants::breach::Breach;

// The first breach any node of a cluster reports: a clone shares it, so each
// node's observer reports into it and whoever runs the cluster reads it.
#[derive(Debug, Clone, Default)]
pub struct Breaches {
    first: Arc<Mutex<Option<Breach>>>,
}

impl Breaches {
    #[must_use]
    pub fn first(&self) -> Option<Breach> {
        self.first.lock().unwrap().clone()
    }

    pub fn report(&self, breach: Breach) {
        self.first.lock().unwrap().get_or_insert(breach);
    }
}
