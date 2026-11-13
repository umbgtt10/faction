// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;

const LENGTH: usize = 4;
const SENDER: usize = 8;

// The bytes a message travels as over TCP and gRPC: a frame, its length -- of
// what follows it -- then the sender, a tag and the payload.
pub struct WireCodec;

impl WireCodec {
    #[must_use]
    pub fn decode(frame: &[u8]) -> Option<TransportMessage> {
        let length = u32::from_le_bytes(frame.get(..LENGTH)?.try_into().ok()?);
        let body = frame.get(LENGTH..)?;
        let from = PeerId::from_le_bytes(body.get(..SENDER)?.try_into().ok()?);
        let tag = *body.get(SENDER)?;
        let whole = usize::try_from(length).ok()? == body.len();
        match (whole, tag) {
            (true, 0) => Some(TransportMessage::Ping { from }),
            (true, 1) => Some(TransportMessage::Ready { from }),
            (true, 2) => Some(TransportMessage::Bootstrapped { from }),
            _ => None,
        }
    }

    #[must_use]
    pub fn encode(message: &TransportMessage) -> Vec<u8> {
        let (from, tag): (PeerId, u8) = match message {
            TransportMessage::Ping { from } => (*from, 0),
            TransportMessage::Ready { from } => (*from, 1),
            TransportMessage::Bootstrapped { from } => (*from, 2),
        };
        let body: Vec<u8> = from.to_le_bytes().into_iter().chain([tag]).collect();
        let length = u32::try_from(body.len()).expect("a frame shorter than 4 GiB");
        length.to_le_bytes().into_iter().chain(body).collect()
    }
}
