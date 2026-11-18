// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;

use crate::infra::logging::shared_writer::SharedWriter;
use crate::traits::fault_report::FaultReport;
use crate::traits::wall_clock::WallClock;

// Every fault one line of the node's log, stamped as its other lines are, so
// a consolidated log reads the faults in their place.
pub struct SharedFileFaultReport {
    writer: SharedWriter,
    clock: Arc<dyn WallClock>,
}

impl SharedFileFaultReport {
    #[must_use]
    pub fn new(writer: SharedWriter, clock: Arc<dyn WallClock>) -> Self {
        Self { writer, clock }
    }
}

impl FaultReport for SharedFileFaultReport {
    fn report(&mut self, fault: &str, from: PeerId, to: PeerId, message: &TransportMessage) {
        let timestamp = self.clock.now().to_stamp();
        self.writer.write_line(&format!(
            "{{\"timestamp\":\"{timestamp}\",\"peer_id\":{from},\"event\":\"fault\",\"fault\":\"{fault}\",\"to\":{to},\"message\":\"{message:?}\"}}"
        ));
    }
}
