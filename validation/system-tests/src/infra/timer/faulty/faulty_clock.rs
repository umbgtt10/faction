// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crate::infra::live_policy::LivePolicy;
use crate::infra::timer::faulty::clock_fault::ClockFault;
use crate::traits::monotonic_clock::MonotonicClock;

const FAITHFUL_PPM: i128 = 1_000_000;

// Where the next reading takes from: the inner clock's reading and this
// one's, unrounded, at the last reading, and the last reading given.
#[derive(Debug, Clone, Copy)]
struct ClockAnchor {
    pub inner: Duration,
    pub read: Duration,
    pub last: Duration,
}

// A [`MonotonicClock`] decorator that reads another at a wrong rate, and in
// coarse ticks, per a [`ClockFault`] that can change while it runs: an
// oscillator drifting, or a tick set for the wrong core clock. Each reading
// takes from the one before it, at the drift the fault states now, so a
// change of rate never moves what was read, and a reading never runs back.
pub struct FaultyClock {
    inner: Arc<dyn MonotonicClock>,
    fault: LivePolicy<ClockFault>,
    anchor: Mutex<ClockAnchor>,
}

impl FaultyClock {
    #[must_use]
    pub fn new(fault: LivePolicy<ClockFault>, inner: Arc<dyn MonotonicClock>) -> Self {
        let start = inner.now();
        Self {
            inner,
            fault,
            anchor: Mutex::new(ClockAnchor {
                inner: start,
                read: start,
                last: start,
            }),
        }
    }

    fn grained(read: Duration, grain: Duration) -> Duration {
        if grain.is_zero() {
            return read;
        }
        let grains = read.as_nanos() / grain.as_nanos();
        Duration::from_nanos(u64::try_from(grains * grain.as_nanos()).unwrap_or(u64::MAX))
    }

    fn scaled(elapsed: Duration, drift_ppm: i64) -> Duration {
        let rate = (FAITHFUL_PPM + i128::from(drift_ppm)).max(0);
        let elapsed = i128::try_from(elapsed.as_nanos()).unwrap_or(i128::MAX);
        let nanos = elapsed.saturating_mul(rate) / FAITHFUL_PPM;
        Duration::from_nanos(u64::try_from(nanos).unwrap_or(u64::MAX))
    }
}

impl MonotonicClock for FaultyClock {
    fn now(&self) -> Duration {
        let fault = self.fault.current();
        let inner = self.inner.now();
        let mut anchor = self.anchor.lock().unwrap();
        let read = anchor.read + Self::scaled(inner.saturating_sub(anchor.inner), fault.drift_ppm);
        let reading = Self::grained(read, fault.grain).max(anchor.last);
        *anchor = ClockAnchor {
            inner,
            read,
            last: reading,
        };
        reading
    }
}
