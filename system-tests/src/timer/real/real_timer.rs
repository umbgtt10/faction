// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::Duration;
use std::time::Instant;

use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_trait::Timer;

const DEFAULT_DELAY: Duration = Duration::from_millis(1);

type Entry = (Reverse<Instant>, TimerEvent);

pub struct RealTimer {
    events: BinaryHeap<Entry>,
}

impl RealTimer {
    pub fn new() -> Self {
        Self {
            events: BinaryHeap::new(),
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
        let deadline = Instant::now() + DEFAULT_DELAY;
        self.events.push((Reverse(deadline), event));
    }

    fn cancel(&mut self, event: TimerEvent) {
        self.events.retain(|(_, e)| *e != event);
    }
}
