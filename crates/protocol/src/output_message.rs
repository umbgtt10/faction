// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::Epoch;
use faction::types::PeerId;

use crate::timer_event::TimerEvent;
use crate::transport_message::TransportMessage;

#[derive(Debug, Clone, PartialEq)]
pub enum OutputMessage {
    BroadcastPing {
        epoch: Epoch,
    },
    BroadcastReady {
        epoch: Epoch,
    },
    Schedule(TimerEvent),
    Cancel(TimerEvent),
    Send {
        to: PeerId,
        message: TransportMessage,
    },
    ProposeAddition {
        peer_id: PeerId,
        epoch: Epoch,
    },
    ProposeRemoval {
        peer_id: PeerId,
        epoch: Epoch,
    },
    Noop,
}
