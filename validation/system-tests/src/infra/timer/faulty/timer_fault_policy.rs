// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// One `0..=100` percentage per firing's fault and per cancel's, the runtime's
// choice on a reschedule -- replace the pending firing, or add one beside it
// -- the timer side's health, and the seed that makes every misbehaving
// decision reproducible. The default is a faithful runtime.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimerFaultPolicy {
    pub late: u8,
    pub reorder: u8,
    pub duplicate: u8,
    pub lost_cancel: u8,
    pub replace: bool,
    pub starved: bool,
    pub dead: bool,
    pub seed: u64,
}
