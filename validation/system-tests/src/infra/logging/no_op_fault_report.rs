// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport_message::TransportMessage;

use crate::traits::fault_report::FaultReport;

// No fault announced: a run that injects thousands, and replays its seed when
// it needs to be read.
pub struct NoOpFaultReport;

impl FaultReport for NoOpFaultReport {
    fn report(&mut self, _: &str, _: u64, _: u64, _: &TransportMessage) {}
}
