// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

pub const DEFAULT_DEADLINE_CYCLES: u32 = 100_000;

pub const NODE_LIFETIME: Duration = Duration::from_secs(30);

// How many of the members a missed probe lists a node asks to probe it.
pub const HELPERS: usize = 2;

// How many probe periods running a member must miss before it is suspected.
pub const SUSPICION_THRESHOLD: usize = 1;
