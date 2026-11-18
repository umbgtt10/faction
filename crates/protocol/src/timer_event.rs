// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::timer_message::TimerMessage;
use faction::instance::peer_id::PeerId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimerEvent<P: PeerId = u64> {
    Fire(TimerMessage<P>),
}
