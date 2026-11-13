// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;

// The change in flight (§4, §11, §12): none, an addition requested of a peer or
// acknowledged by it, or the removal of a peer requested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reconfiguration {
    None,
    AdditionRequested(String),
    AdditionAcknowledged(String),
    RemovalRequested(String),
}
