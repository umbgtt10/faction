// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

pub trait Pause: Send + Sync {
    fn pause(&self, duration: Duration);
}
