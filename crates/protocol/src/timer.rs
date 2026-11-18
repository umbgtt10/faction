// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::timer_event::TimerEvent;
use faction::instance::peer_id::PeerId;

pub trait Timer<P: PeerId = u64>: Send {
    fn poll(&mut self) -> Option<TimerEvent<P>>;

    fn schedule(&mut self, event: TimerEvent<P>);

    fn cancel(&mut self, event: TimerEvent<P>);
}
