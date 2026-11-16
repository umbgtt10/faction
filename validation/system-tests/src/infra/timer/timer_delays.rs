// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

// A node's timer delays: its base delay, its deadline's, and how many times
// over it waits the probe period and its timeouts; and how long a node that
// runs on its own -- a thread's, a process's -- runs before it stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerDelays {
    pub delay: Duration,
    pub deadline_delay: Duration,
    pub probe_stretch: u32,
    pub lifetime: Duration,
}
