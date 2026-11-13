// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use chrono::DateTime;
use chrono::Utc;

use crate::traits::wall_clock::WallClock;

pub struct SystemWallClock;

impl WallClock for SystemWallClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
