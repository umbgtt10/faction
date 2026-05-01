// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::VecDeque;

use faction_protocol::timer_event::TimerEvent;

use crate::timer::timer_trait::Timer;

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
