// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::infra::timer::wall_time::WallTime;

pub trait WallClock: Send + Sync {
    fn now(&self) -> WallTime;
}
