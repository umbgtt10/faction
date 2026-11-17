// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use serde_json::Value;
use serde_json::json;

use crate::answer::Answer;
use crate::configuration::Configuration;
use crate::epoch::Epoch;
use crate::node_state::NodeState;
use crate::provisional::Provisional;
use crate::reconfiguration::Reconfiguration;
use crate::rules::rule_book::RuleBook;
use crate::spec_reader::SpecReader;
use crate::table::decision_table::DecisionTable;
use crate::view::View;
use crate::vocabulary::stage::Stage;

// A node as the specification states it: created from its configuration, and
// answering each command, given in the specification's encoding, with a
// receipt -- the result the first row of the decision table that applies
// gives it, numbered one more than the last.
pub struct Model {
    table: DecisionTable,
    state: NodeState,
    next_number: u64,
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
                reconfiguration: Reconfiguration::None,
                completed_before_removal: false,
                suspicion_threshold: configuration.suspicion_threshold,
                suspected: Vec::new(),
                misses: BTreeMap::new(),
                last_probed: None,
                epoch: configuration.epoch,
                provisional: configuration.provisional,
            },
            next_number: 1,
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
            epoch: Epoch::after(&command, row.result, changed.epoch),
            provisional: Provisional::after(&command, &row.outcomes, &self.state),
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
            receipt: json!({
                "number": self.next_number.to_string(),
                "output": row.result.encoded(outcomes, View::of(&state), admissible),
            }),
        };
        self.state = state;
        self.next_number += 1;
        answer
    }
}
