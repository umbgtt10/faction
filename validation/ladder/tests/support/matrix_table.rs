// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::spec_reader::SpecReader;
use faction::command::Command;
use serde_json::json;
use serde_json::Value;

// The specification's decision table, read by the matrix tier: what each stage
// admits, and each row's stages, commands, result, outcomes and next stage --
// so the tier's expectations are the table's, and the two cannot disagree
// unnoticed.
pub struct MatrixTable {
    table: Value,
}

impl MatrixTable {
    pub fn new(table: Value) -> Self {
        Self { table }
    }

    // The commands a stage admits, as the crate lists them: in the commands'
    // order, with the placeholder the crate gives a command that carries a peer
    // id or a required count -- zero, either way.
    pub fn admits(&self, stage: &str) -> Vec<Command> {
        self.table["admits"][stage]
            .as_array()
            .unwrap_or_else(|| panic!("the table has no stage {stage}"))
            .iter()
            .map(|name| {
                let name = name.as_str().expect("a command's name");
                SpecReader::command(&json!({ name: { "peer_id": "0", "required_count": "0" } }))
            })
            .collect()
    }

    // Where a call, made in the stage `before` and answered `result` -- both as
    // the specification writes them -- departs from the row `id`: nothing when
    // that row decided it.
    pub fn departures(
        &self,
        id: &str,
        before: &str,
        command: &Value,
        result: &Value,
    ) -> Vec<String> {
        let row = self.row(id);
        let command = Self::name(command);
        let (kind, body) = Self::member(result);
        let outcomes: Vec<&str> = body["outcomes"]
            .as_array()
            .map(|outcomes| outcomes.iter().map(Self::name).collect())
            .unwrap_or_default();
        let after = self.stage_of(result);
        let next = match row["next"].as_str().expect("a row's next stage") {
            "unchanged" => before,
            stage => stage,
        };
        let mut departures = Vec::new();
        if !Self::names(&row["stages"]).contains(&before) {
            departures.push(format!("{id} does not apply in {before}"));
        }
        if !Self::names(&row["commands"]).contains(&command) {
            departures.push(format!("{id} does not apply to {command}"));
        }
        if row["result"].as_str() != Some(kind) {
            departures.push(format!("{id} answers {}, not {kind}", row["result"]));
        }
        if Self::names(&row["outcomes"]) != outcomes {
            departures.push(format!("{id} gives {}, not {outcomes:?}", row["outcomes"]));
        }
        if next != after {
            departures.push(format!("{id} leaves the node {next}, not {after}"));
        }
        departures
    }

    // The stage a result was answered in, known by what it admits.
    pub fn stage_of(&self, result: &Value) -> String {
        let (_, body) = Self::member(result);
        self.table["admits"]
            .as_object()
            .expect("the table's admission")
            .iter()
            .find(|(_, admitted)| **admitted == body["admissible"])
            .map(|(stage, _)| stage.clone())
            .unwrap_or_else(|| panic!("no stage admits {}", body["admissible"]))
    }

    fn member(value: &Value) -> (&str, &Value) {
        value
            .as_object()
            .and_then(|object| object.iter().next())
            .map(|(name, body)| (name.as_str(), body))
            .expect("an enumeration's value is an object of one member")
    }

    fn name(value: &Value) -> &str {
        Self::member(value).0
    }

    fn names(value: &Value) -> Vec<&str> {
        value
            .as_array()
            .expect("a list of names")
            .iter()
            .map(|name| name.as_str().expect("a name"))
            .collect()
    }

    fn row(&self, id: &str) -> &Value {
        self.table["rows"]
            .as_array()
            .expect("the table's rows")
            .iter()
            .find(|row| row["id"] == id)
            .unwrap_or_else(|| panic!("the table has no row {id}"))
    }
}
