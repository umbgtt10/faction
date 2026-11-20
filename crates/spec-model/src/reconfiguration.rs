// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;

// A change in flight (§4, §11, §12): an addition requested of a peer or
// acknowledged by it, or the removal of a peer requested, with the count its
// request stated it leaves, if it stated one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reconfiguration {
    AdditionRequested(String),
    AdditionAcknowledged(String),
    RemovalRequested(String, Option<u64>),
}
