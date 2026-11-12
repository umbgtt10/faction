// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::timer_event::TimerEvent;

pub trait Timer: Send {
    fn poll(&mut self) -> Option<TimerEvent>;

    fn schedule(&mut self, event: TimerEvent);

    fn cancel(&mut self, event: TimerEvent);
}
