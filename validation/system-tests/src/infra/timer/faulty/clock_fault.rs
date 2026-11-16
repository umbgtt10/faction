// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

// How a node's clock runs from true: its drift in parts per million -- above
// 0 fast, below slow, a million below or more stopped -- and the grain of its
// ticks, none at zero. The default is a faithful clock.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClockFault {
    pub drift_ppm: i64,
    pub grain: Duration,
}
