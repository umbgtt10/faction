// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::PeerId;

use crate::transport::transport_trait::Transport;
use faction_protocol::transport_message::TransportMessage;

pub struct InMemoryTransport;

impl Transport for InMemoryTransport {
    fn send(&mut self, _to: PeerId, _message: TransportMessage) {}

    fn recv(&mut self) -> Option<(PeerId, TransportMessage)> {
        None
    }
}
