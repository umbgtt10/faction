// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::timer_event::TimerEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum OutputMessage {
    BroadcastPing,
    BroadcastReady,
    Schedule(TimerEvent),
    Cancel(TimerEvent),
    Noop,
}
