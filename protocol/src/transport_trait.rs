// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;

use crate::transport_message::TransportMessage;

pub trait Transport: Send {
    fn send(&mut self, to: PeerId, message: TransportMessage);

    fn recv(&mut self) -> Option<TransportMessage>;
}
