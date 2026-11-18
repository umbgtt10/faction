// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::instance::peer_id::PeerId;
use faction::types::Epoch;

use crate::timer_event::TimerEvent;
use crate::transport_message::TransportMessage;

#[derive(Debug, Clone, PartialEq)]
pub enum OutputMessage<P: PeerId = u64> {
    BroadcastPing { epoch: Epoch },
    BroadcastReady { epoch: Epoch },
    Schedule(TimerEvent<P>),
    Cancel(TimerEvent<P>),
    Send { to: P, message: TransportMessage<P> },
    ProposeAddition { peer_id: P, epoch: Epoch },
    ProposeRemoval { peer_id: P, epoch: Epoch },
    Noop,
}
