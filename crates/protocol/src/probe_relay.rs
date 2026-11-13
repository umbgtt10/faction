// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use faction::types::PeerId;

use crate::output_message::OutputMessage;
use crate::transport_message::TransportMessage;

// What a node does with a probe on the wire, its own Faction told nothing: it
// answers a probe; probes a target on behalf of the node that asked it to; and
// relays the target's answer back to that node. The probe and its answer carry
// whom they are for, so a helper keeps no table of what it relays.
pub struct ProbeRelay;

impl ProbeRelay {
    #[must_use]
    pub fn reply(local_peer_id: PeerId, message: &TransportMessage) -> Vec<OutputMessage> {
        match *message {
            TransportMessage::Probe { from, relay_for } => vec![OutputMessage::Send {
                to: from,
                message: TransportMessage::ProbeAnswer {
                    from: local_peer_id,
                    relay_for,
                },
            }],
            TransportMessage::IndirectProbe { from, target } => vec![OutputMessage::Send {
                to: target,
                message: TransportMessage::Probe {
                    from: local_peer_id,
                    relay_for: Some(from),
                },
            }],
            TransportMessage::ProbeAnswer {
                from,
                relay_for: Some(asker),
            } => vec![OutputMessage::Send {
                to: asker,
                message: TransportMessage::IndirectProbeAnswer {
                    from: local_peer_id,
                    target: from,
                },
            }],
            _ => vec![OutputMessage::Noop],
        }
    }
}
