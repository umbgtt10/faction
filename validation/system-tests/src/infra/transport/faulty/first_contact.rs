// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;

// The first messages to each peer, lost to the neighbour resolution a first
// contact waits on: counted per peer, so contact once made stays made.
#[derive(Debug, Default)]
pub struct FirstContact {
    sent: BTreeMap<u64, u8>,
}

impl FirstContact {
    // Whether a message to `to` is among the first `lost` sent to it.
    pub fn loses(&mut self, to: u64, lost: u8) -> bool {
        let sent = self.sent.entry(to).or_insert(0);
        let loses = *sent < lost;
        *sent = sent.saturating_add(1);
        loses
    }
}
