// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::collections::VecDeque;

use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_trait::Timer;

pub struct InMemoryTimer {
    events: VecDeque<TimerEvent>,
}

impl InMemoryTimer {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }
}

impl Default for InMemoryTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer for InMemoryTimer {
    fn poll(&mut self) -> Option<TimerEvent> {
        self.events.pop_front()
    }

    fn schedule(&mut self, event: TimerEvent) {
        self.events.push_back(event);
    }

    fn cancel(&mut self, event: TimerEvent) {
        self.events.retain(|e| *e != event);
    }
}
