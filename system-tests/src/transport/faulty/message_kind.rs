// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport_message::TransportMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Ping,
    Ready,
    Bootstrapped,
}

impl MessageKind {
    #[must_use]
    pub fn of(message: &TransportMessage) -> MessageKind {
        match message {
            TransportMessage::Ping { .. } => MessageKind::Ping,
            TransportMessage::Ready { .. } => MessageKind::Ready,
            TransportMessage::Bootstrapped { .. } => MessageKind::Bootstrapped,
        }
    }
}
