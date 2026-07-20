// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::Duration;
use std::time::Instant;

use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_trait::Timer;

type Entry = (Reverse<Instant>, TimerEvent);

pub struct RealTimer {
    events: BinaryHeap<Entry>,
    delay: Duration,
}

impl RealTimer {
    pub fn new() -> Self {
        Self {
            events: BinaryHeap::new(),
            delay: Duration::from_millis(50),
        }
    }

    pub fn with_delay(delay: Duration) -> Self {
        Self {
            events: BinaryHeap::new(),
            delay,
        }
    }
}

impl Default for RealTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer for RealTimer {
    fn poll(&mut self) -> Option<TimerEvent> {
        match self.events.peek() {
            Some((Reverse(deadline), _)) if *deadline <= Instant::now() => {
                self.events.pop().map(|(_, event)| event)
            }
            _ => None,
        }
    }

    fn schedule(&mut self, event: TimerEvent) {
        let deadline = Instant::now() + self.delay;
        self.events.push((Reverse(deadline), event));
    }

    fn cancel(&mut self, event: TimerEvent) {
        self.events.retain(|(_, e)| *e != event);
    }
}
