// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::timer_message::TimerMessage;
use crate::transport_message::TransportMessage;
use faction::instance::peer_id::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMessage<P: PeerId = u64> {
    Transport(TransportMessage<P>),
    Timer(TimerMessage<P>),
}
