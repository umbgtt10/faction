// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::timer_message::TimerMessage;
use crate::transport_message::TransportMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMessage {
    Transport(TransportMessage),
    Timer(TimerMessage),
}
