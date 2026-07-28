// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use super::join_context::JoinContext;
use super::process_join_context::ProcessJoinContext;

pub enum Joining {
    InProcess(JoinContext),
    Process(ProcessJoinContext),
}
