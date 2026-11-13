// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;

use crate::timer_event::TimerEvent;
use crate::transport_message::TransportMessage;

#[derive(Debug, Clone, PartialEq)]
pub enum OutputMessage {
    BroadcastPing,
    BroadcastReady,
    Schedule(TimerEvent),
    Cancel(TimerEvent),
    Send {
        to: PeerId,
        message: TransportMessage,
    },
    ProposeAddition {
        peer_id: PeerId,
    },
    ProposeRemoval {
        peer_id: PeerId,
    },
    Noop,
}
