// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// What a node restarts from: its genesis, or a snapshot of a membership it
// held, `behind` commits short of the last it applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartFrom {
    Genesis,
    Snapshot { behind: usize },
}
