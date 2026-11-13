// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    pub peer_id: String,
    pub peers: Vec<String>,
    pub required_count: u64,
    pub suspicion_threshold: u64,
}
