// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// What a run left the counter showing, in bytes counted from before the
// machine was created: what it held new, what it held after the run's last
// command, and what was left once it was dropped.
#[derive(Debug)]
pub struct MemoryProfile {
    pub skeleton: isize,
    pub held_after_all: isize,
    pub after_drop: isize,
}
