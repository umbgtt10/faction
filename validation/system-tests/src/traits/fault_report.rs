// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport_message::TransportMessage;

// Where a fault layer announces a fault it injected: which fault, on the
// link from which node to which, and the message it struck.
pub trait FaultReport: Send {
    fn report(&mut self, fault: &str, from: u64, to: u64, message: &TransportMessage);
}
