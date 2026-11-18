// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::instance::peer_id::PeerId;

use crate::transport_message::TransportMessage;

pub trait Transport<P: PeerId = u64>: Send {
    fn send(&mut self, to: P, message: TransportMessage<P>);

    fn recv(&mut self) -> Option<TransportMessage<P>>;
}
