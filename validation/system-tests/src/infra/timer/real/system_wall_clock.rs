// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::infra::timer::wall_time::WallTime;
use crate::traits::wall_clock::WallClock;

pub struct SystemWallClock;

// The system's clock, read as the time since the epoch: a clock set before it
// reads the epoch.
impl WallClock for SystemWallClock {
    fn now(&self) -> WallTime {
        WallTime::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default(),
        )
    }
}
