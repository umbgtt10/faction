// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// What the layer that commits changes -- the harness, in the system tests --
// does with an addition once the newcomer has acknowledged its membership:
// commits it everywhere, a new required count with it if one is given, or
// aborts it on the coordinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Committer {
    Commit { required_count: Option<usize> },
    Abort,
}
