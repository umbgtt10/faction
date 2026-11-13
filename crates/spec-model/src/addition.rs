// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;

// The addition in flight (§11): none, requested of a peer, or acknowledged by
// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Addition {
    None,
    Requested(String),
    Acknowledged(String),
}
