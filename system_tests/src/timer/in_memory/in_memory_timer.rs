// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol::timer_event::TimerEvent;

use crate::timer::timer_trait::Timer;

pub struct InMemoryTimer;

impl Timer for InMemoryTimer {
    fn poll(&mut self) -> Option<TimerEvent> {
        None
    }

    fn schedule(&mut self, _event: TimerEvent) {}

    fn cancel(&mut self, _event: TimerEvent) {}
}
