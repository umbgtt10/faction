// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::PeerId;

use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;

pub struct TcpTransport;

impl Transport for TcpTransport {
    fn send(&mut self, _to: PeerId, _message: TransportMessage) {}

    fn recv(&mut self) -> Option<TransportMessage> {
        None
    }
}
