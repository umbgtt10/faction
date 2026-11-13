// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

pub trait MonotonicClock: Send + Sync {
    fn now(&self) -> Duration;
}
