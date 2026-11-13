// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

pub const DEFAULT_DEADLINE_CYCLES: u32 = 100_000;

pub const NODE_LIFETIME: Duration = Duration::from_secs(30);

// How many of the members a missed probe lists a node asks to probe it.
pub const HELPERS: usize = 2;

// How many probe periods running a member must miss before it is suspected,
// unless a test gives its cluster another threshold.
pub const DEFAULT_SUSPICION_THRESHOLD: usize = 1;

// How many times over the probe period and its timeouts are waited, unless a
// test stretches them for nodes that cannot keep up at their own.
pub const DEFAULT_PROBE_STRETCH: u32 = 1;
