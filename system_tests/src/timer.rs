// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::message::TimerMessage;

#[derive(Debug, Clone)]
pub enum TimerEvent {
    Fire(TimerMessage),
}

pub trait Timer: Send {
    fn poll(&mut self) -> Option<TimerEvent>;

    fn schedule(&mut self, event: TimerEvent);

    fn cancel(&mut self, event: TimerEvent);
}
