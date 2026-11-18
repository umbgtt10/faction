// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::timer::Timer;
use faction_protocol::timer_event::TimerEvent;

use crate::infra::dice::Dice;
use crate::infra::live_policy::LivePolicy;
use crate::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;

const DUPLICATE_POLLS: u64 = 2;
const LATE_POLLS: u64 = 8;
const REORDER_MAX_POLLS: u64 = 4;
const SEED_MIX: u64 = 0x9E37_79B9_7F4A_7C15;

// A [`Timer`] decorator that keeps the runtime's contract -- a timer
// scheduled fires at least once -- as loosely as a runtime may, per a seeded
// [`TimerFaultPolicy`] that can change while it runs: a firing held late, or
// overtaken by a later one, or delivered twice; a cancel that lost its race;
// a reschedule that replaces the pending firing. And beyond the contract, a
// timer side starved, its firings kept for when it is fed, or dead, every
// firing lost until it is healed. Lags are counted in the node's polls of it.
pub struct FaultyTimer {
    inner: Box<dyn Timer>,
    policy: LivePolicy<TimerFaultPolicy>,
    dice: Dice,
    polls: u64,
    held: Vec<(u64, TimerEvent)>,
}

impl FaultyTimer {
    #[must_use]
    pub fn new(peer_id: u64, policy: LivePolicy<TimerFaultPolicy>, inner: Box<dyn Timer>) -> Self {
        let dice = Dice::new(policy.current().seed ^ peer_id.wrapping_mul(SEED_MIX));
        Self {
            inner,
            policy,
            dice,
            polls: 0,
            held: Vec::new(),
        }
    }

    // A firing come due, delivered now, held for later, or both.
    fn deliver(&mut self, event: TimerEvent, policy: &TimerFaultPolicy) -> Option<TimerEvent> {
        if self.dice.rolls(policy.late) {
            self.hold(LATE_POLLS, event);
            return None;
        }
        if self.dice.rolls(policy.reorder) {
            let lag = 1 + self.dice.below(REORDER_MAX_POLLS);
            self.hold(lag, event);
            return None;
        }
        if self.dice.rolls(policy.duplicate) {
            self.hold(DUPLICATE_POLLS, event.clone());
        }
        Some(event)
    }

    // The first held firing whose lag has run out.
    fn due(&mut self) -> Option<TimerEvent> {
        let index = self.held.iter().position(|(due, _)| *due <= self.polls)?;
        Some(self.held.remove(index).1)
    }

    fn hold(&mut self, lag: u64, event: TimerEvent) {
        self.held.push((self.polls + lag, event));
    }

    // Every pending firing of `event` withdrawn, held ones among them.
    fn withdraw(&mut self, event: &TimerEvent) {
        self.inner.cancel(event.clone());
        self.held.retain(|(_, held)| held != event);
    }
}

impl Timer for FaultyTimer {
    fn poll(&mut self) -> Option<TimerEvent> {
        let policy = self.policy.current();
        if policy.dead {
            while self.inner.poll().is_some() {}
            self.held.clear();
            return None;
        }
        if policy.starved {
            return None;
        }
        self.polls += 1;
        if let Some(event) = self.due() {
            return Some(event);
        }
        let event = self.inner.poll()?;
        self.deliver(event, &policy)
    }

    fn schedule(&mut self, event: TimerEvent) {
        if self.policy.current().replace {
            self.withdraw(&event);
        }
        self.inner.schedule(event);
    }

    fn cancel(&mut self, event: TimerEvent) {
        if self.dice.rolls(self.policy.current().lost_cancel) {
            return;
        }
        self.withdraw(&event);
    }
}
