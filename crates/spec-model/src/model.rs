// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;
use serde_json::Value;

use crate::answer::Answer;
use crate::configuration::Configuration;
use crate::node_state::NodeState;
use crate::rules::rule_book::RuleBook;
use crate::spec_reader::SpecReader;
use crate::table::decision_table::DecisionTable;
use crate::view::View;
use crate::vocabulary::stage::Stage;

// A node as the specification states it: created from its configuration, and
// answering each command, given in the specification's encoding, with the
// result the first row of the decision table that applies gives it.
pub struct Model {
    table: DecisionTable,
    state: NodeState,
}

impl Model {
    #[must_use]
    pub fn new(table: DecisionTable, configuration: Configuration) -> Self {
        Self {
            table,
            state: NodeState {
                peer_id: configuration.peer_id,
                required_count: configuration.required_count,
                stage: Stage::Initial,
                members: configuration.peers,
                pinging_peers: Vec::new(),
                collecting_peers: Vec::new(),
                deadline_missed: false,
            },
        }
    }

    pub fn process(&mut self, command: &Value) -> Answer {
        let command = SpecReader::command(command);
        let row = self.table.row(&command, &self.state);
        let outcomes: Vec<Value> = row
            .outcomes
            .iter()
            .map(|outcome| outcome.encoded(&command, &self.state))
            .collect();
        let changed = RuleBook::rule(&row.id)
            .change
            .applied(&command, &self.state);
        let state = NodeState {
            stage: row.next.unwrap_or(changed.stage),
            ..changed
        };
        let admissible: Value = self
            .table
            .admits(state.stage)
            .iter()
            .map(|name| Value::from(name.name()))
            .collect();
        let answer = Answer {
            row: String::from(&row.id),
            result: row.result.encoded(outcomes, View::of(&state), admissible),
        };
        self.state = state;
        answer
    }
}
