// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::timer_message::TimerMessage;
use crate::transport_message::TransportMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMessage {
    Transport(TransportMessage),
    Timer(TimerMessage),
}
