// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::Duration;
use std::time::Instant;

use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::timer_trait::Timer;

pub const DEFAULT_DEADLINE_CYCLES: u32 = 100_000;

type Entry = (Reverse<Instant>, TimerEvent);

pub struct RealTimer {
    events: BinaryHeap<Entry>,
    delay: Duration,
    deadline_delay: Duration,
}

impl RealTimer {
    pub fn new() -> Self {
        Self::with_delay(Duration::from_millis(50))
    }

    pub fn with_delay(delay: Duration) -> Self {
        Self::with_delays(delay, delay * DEFAULT_DEADLINE_CYCLES)
    }

    pub fn with_delays(delay: Duration, deadline_delay: Duration) -> Self {
        Self {
            events: BinaryHeap::new(),
            delay,
            deadline_delay,
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
        let delay = if matches!(event, TimerEvent::Fire(TimerMessage::DeadlineExpired)) {
            self.deadline_delay
        } else {
            self.delay
        };
        self.events.push((Reverse(Instant::now() + delay), event));
    }

    fn cancel(&mut self, event: TimerEvent) {
        self.events.retain(|(_, e)| *e != event);
    }
}
