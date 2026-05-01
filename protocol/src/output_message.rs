// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::timer_event::TimerEvent;

#[derive(Debug, Clone)]
pub enum OutputMessage {
    BroadcastReady,
    Schedule(TimerEvent),
    Cancel(TimerEvent),
    Noop,
}
