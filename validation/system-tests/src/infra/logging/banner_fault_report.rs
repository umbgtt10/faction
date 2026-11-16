// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::io::Write;

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;

use crate::traits::fault_report::FaultReport;

const RULE: &str = "###############################################################";

// Every fault a loud banner on `out` -- stderr, for a node with no log of its
// own -- so a test that fails shows what was injected.
pub struct BannerFaultReport {
    out: Box<dyn Write + Send>,
}

impl BannerFaultReport {
    #[must_use]
    pub fn new(out: Box<dyn Write + Send>) -> Self {
        Self { out }
    }
}

impl FaultReport for BannerFaultReport {
    fn report(&mut self, fault: &str, from: PeerId, to: PeerId, message: &TransportMessage) {
        let _ = write!(
            self.out,
            "\n{RULE}\n###  FAULT INJECTED >>> {fault}\n###  peer {from} -> peer {to}   {message:?}\n{RULE}\n\n"
        );
        let _ = self.out.flush();
    }
}
