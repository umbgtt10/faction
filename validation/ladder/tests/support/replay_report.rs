// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// What a replay made of a log: how many steps it held, how many the replay
// answered as logged, and where it answered otherwise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayReport {
    pub lines: usize,
    pub reconstructed: usize,
    pub departures: Vec<String>,
}
