// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Duration;

use faction_protocol::timer::Timer;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;

use crate::constants::DEFAULT_DEADLINE_CYCLES;
use crate::traits::monotonic_clock::MonotonicClock;

const DEFAULT_DELAY: Duration = Duration::from_millis(50);

type Entry = (Reverse<Duration>, Reverse<u64>, TimerEvent);

pub struct RealTimer {
    clock: Arc<dyn MonotonicClock>,
    events: BinaryHeap<Entry>,
    scheduled: u64,
    delay: Duration,
    deadline_delay: Duration,
}

impl RealTimer {
    #[must_use]
    pub fn new(clock: Arc<dyn MonotonicClock>) -> Self {
        Self::with_delay(clock, DEFAULT_DELAY)
    }

    #[must_use]
    pub fn with_delay(clock: Arc<dyn MonotonicClock>, delay: Duration) -> Self {
        Self::with_delays(clock, delay, delay * DEFAULT_DEADLINE_CYCLES)
    }

    #[must_use]
    pub fn with_delays(
        clock: Arc<dyn MonotonicClock>,
        delay: Duration,
        deadline_delay: Duration,
    ) -> Self {
        Self {
            clock,
            events: BinaryHeap::new(),
            scheduled: 0,
            delay,
            deadline_delay,
        }
    }
}

impl Timer for RealTimer {
    fn poll(&mut self) -> Option<TimerEvent> {
        match self.events.peek() {
            Some((Reverse(deadline), _, _)) if *deadline <= self.clock.now() => {
                self.events.pop().map(|(_, _, event)| event)
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
        self.scheduled += 1;
        self.events.push((
            Reverse(self.clock.now() + delay),
            Reverse(self.scheduled),
            event,
        ));
    }

    fn cancel(&mut self, event: TimerEvent) {
        self.events.retain(|(_, _, scheduled)| *scheduled != event);
    }
}
