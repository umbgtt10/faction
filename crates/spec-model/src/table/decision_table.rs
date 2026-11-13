// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::command::Command;
use crate::node_state::NodeState;
use crate::rules::rule_book::RuleBook;
use crate::table::row::Row;
use crate::vocabulary::command_name::CommandName;
use crate::vocabulary::stage::Stage;

// The decision table: what each stage admits, and the rows in the order they
// are checked. A command is answered by the first row that applies (§3).
pub struct DecisionTable {
    admits: BTreeMap<Stage, Vec<CommandName>>,
    rows: Vec<Row>,
}

impl DecisionTable {
    #[must_use]
    pub fn new(admits: BTreeMap<Stage, Vec<CommandName>>, rows: Vec<Row>) -> Self {
        Self { admits, rows }
    }

    #[must_use]
    pub fn admits(&self, stage: Stage) -> &[CommandName] {
        &self.admits[&stage]
    }

    #[must_use]
    pub fn row(&self, command: &Command, state: &NodeState) -> &Row {
        let admitted = self.admits(state.stage);
        self.rows
            .iter()
            .find(|row| {
                row.stages.contains(&state.stage)
                    && row.commands.contains(&command.name)
                    && RuleBook::rule(&row.id)
                        .situation
                        .holds(command, state, admitted)
            })
            .expect("a row that applies: the table answers every command in every stage")
    }
}
