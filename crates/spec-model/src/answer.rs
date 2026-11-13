// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub row: String,
    pub result: Value,
}
