// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, Copy)]
pub enum Spawn {
    Task,
    Thread,
    Process,
}
