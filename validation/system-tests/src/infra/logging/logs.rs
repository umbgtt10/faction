// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

// Where a recipe's nodes log: nowhere but the faults they inject, bannered on
// stderr; one file a node under a folder; or nowhere at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Logs {
    Stderr,
    Files(PathBuf),
    Silent,
}
