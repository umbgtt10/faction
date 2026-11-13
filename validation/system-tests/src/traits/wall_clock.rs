// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use chrono::DateTime;
use chrono::Utc;

pub trait WallClock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}
