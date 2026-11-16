// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;

use crate::infra::logging::shared_writer::SharedWriter;
use crate::traits::fault_report::FaultReport;

// Every fault one line of the node's log.
pub struct SharedFileFaultReport {
    writer: SharedWriter,
}

impl SharedFileFaultReport {
    #[must_use]
    pub fn new(writer: SharedWriter) -> Self {
        Self { writer }
    }
}

impl FaultReport for SharedFileFaultReport {
    fn report(&mut self, fault: &str, from: PeerId, to: PeerId, message: &TransportMessage) {
        self.writer.write_line(&format!(
            "{{\"event\":\"fault\",\"fault\":\"{fault}\",\"node\":{from},\"to\":{to},\"message\":\"{message:?}\"}}"
        ));
    }
}
