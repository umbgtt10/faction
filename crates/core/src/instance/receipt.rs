// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::instance::peer_id::PeerId;
use crate::process_result::ProcessResult;

// Every call's answer: its output, and its number, one more than the
// instance's last receipt's, the first being 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt<P: PeerId = u64> {
    pub number: u64,
    pub output: ProcessResult<P>,
}
