// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportMessage {
    Ping { from: PeerId },
    Ready { from: PeerId },
    Bootstrapped { from: PeerId },
}
