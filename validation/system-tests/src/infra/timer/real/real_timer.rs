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
use crate::constants::DEFAULT_PROBE_STRETCH;
use crate::traits::monotonic_clock::MonotonicClock;

const DEFAULT_DELAY: Duration = Duration::from_millis(50);
// The probe period and its timeouts, in delays, before a test stretches them:
// a period long enough for a node to keep up with the probes it sends and
// answers; a probe answered within two hops' time and a relayed one within
// four, with room to spare.
const INDIRECT_PROBES_DELAYS: u32 = 8;
const PROBE_DUE_DELAYS: u32 = 5;
const PROBE_MISSED_DELAYS: u32 = 4;

type Entry = (Reverse<Duration>, Reverse<u64>, TimerEvent);

pub struct RealTimer {
    clock: Arc<dyn MonotonicClock>,
    events: BinaryHeap<Entry>,
    scheduled: u64,
    delay: Duration,
    deadline_delay: Duration,
    probe_stretch: u32,
}

impl RealTimer {
    #[must_use]
    pub fn new(clock: Arc<dyn MonotonicClock>) -> Self {
        Self::with_delay(clock, DEFAULT_DELAY)
    }

    #[must_use]
    pub fn with_delay(clock: Arc<dyn MonotonicClock>, delay: Duration) -> Self {
        Self::with_delays(
            clock,
            delay,
            delay * DEFAULT_DEADLINE_CYCLES,
            DEFAULT_PROBE_STRETCH,
        )
    }

    // `probe_stretch`: how many times over the probe period and its timeouts
    // are waited, for a node that cannot keep up with them at their own.
    #[must_use]
    pub fn with_delays(
        clock: Arc<dyn MonotonicClock>,
        delay: Duration,
        deadline_delay: Duration,
        probe_stretch: u32,
    ) -> Self {
        Self {
            clock,
            events: BinaryHeap::new(),
            scheduled: 0,
            delay,
            deadline_delay,
            probe_stretch,
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
        let delay = match event {
            TimerEvent::Fire(TimerMessage::DeadlineExpired) => self.deadline_delay,
            TimerEvent::Fire(TimerMessage::ProbeDue) => {
                self.delay * PROBE_DUE_DELAYS * self.probe_stretch
            }
            TimerEvent::Fire(TimerMessage::ProbeMissed { .. }) => {
                self.delay * PROBE_MISSED_DELAYS * self.probe_stretch
            }
            TimerEvent::Fire(TimerMessage::IndirectProbesMissed { .. }) => {
                self.delay * INDIRECT_PROBES_DELAYS * self.probe_stretch
            }
            _ => self.delay,
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
