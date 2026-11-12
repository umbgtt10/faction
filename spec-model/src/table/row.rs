// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::vocabulary::command_name::CommandName;
use crate::vocabulary::outcome_name::OutcomeName;
use crate::vocabulary::result_name::ResultName;
use crate::vocabulary::stage::Stage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub stages: Vec<Stage>,
    pub commands: Vec<CommandName>,
    pub result: ResultName,
    pub outcomes: Vec<OutcomeName>,
    pub next: Option<Stage>,
}
