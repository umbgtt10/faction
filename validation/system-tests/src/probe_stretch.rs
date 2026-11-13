// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// How many times over a node waits the probe period and its timeouts: no more
// than their own for a node that keeps up with its probes, threefold for a
// task's, which reads one input a step.
#[derive(Debug, Clone, Copy)]
pub enum ProbeStretch {
    Minimal,
    Generous,
}

impl ProbeStretch {
    pub fn factor(&self) -> u32 {
        match self {
            ProbeStretch::Minimal => 1,
            ProbeStretch::Generous => 3,
        }
    }
}
