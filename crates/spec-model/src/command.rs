// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::vocabulary::command_name::CommandName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub name: CommandName,
    pub peer_id: Option<String>,
    pub required_count: Option<u64>,
    pub epoch: u64,
    pub members: Vec<String>,
}
