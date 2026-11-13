// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use crate::traits::monotonic_clock::MonotonicClock;
use crate::traits::pause::Pause;
use crate::traits::wall_clock::WallClock;

#[derive(Clone)]
pub struct TimePorts {
    pub clock: Arc<dyn MonotonicClock>,
    pub pause: Arc<dyn Pause>,
    pub wall_clock: Arc<dyn WallClock>,
}
