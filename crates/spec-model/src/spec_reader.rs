// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde_json::Value;

use crate::command::Command;
use crate::configuration::Configuration;
use crate::table::decision_table::DecisionTable;
use crate::table::row::Row;
use crate::vocabulary::command_name::CommandName;
use crate::vocabulary::outcome_name::OutcomeName;
use crate::vocabulary::result_name::ResultName;
use crate::vocabulary::stage::Stage;

const UNCHANGED: &str = "unchanged";

// The specification's encoding read: a command, a configuration, the decision
// table. What it reads is valid against the schema, or outside the
// specification, and a reading of what is outside it panics.
pub struct SpecReader;

impl SpecReader {
    #[must_use]
    pub fn command(value: &Value) -> Command {
        let (name, payload) = value
            .as_object()
            .and_then(|object| object.iter().next())
            .expect("a command: an object of one member");
        Command {
            name: CommandName::of(name),
            peer_id: payload.get("peer_id").map(Self::string),
            required_count: payload.get("required_count").map(Self::count),
        }
    }

    #[must_use]
    pub fn configuration(value: &Value) -> Configuration {
        Configuration {
            peer_id: Self::string(&value["peer_id"]),
            peers: Self::list(&value["peers"]).map(Self::string).collect(),
            required_count: Self::count(&value["required_count"]),
            suspicion_threshold: Self::count(&value["suspicion_threshold"]),
        }
    }

    #[must_use]
    pub fn table(value: &Value) -> DecisionTable {
        let admits: BTreeMap<Stage, Vec<CommandName>> = Stage::ALL
            .into_iter()
            .map(|stage| {
                let admitted = Self::names(&value["admits"][stage.name()])
                    .map(CommandName::of)
                    .collect();
                (stage, admitted)
            })
            .collect();
        let rows = Self::list(&value["rows"]).map(Self::row).collect();
        DecisionTable::new(admits, rows)
    }

    fn count(value: &Value) -> u64 {
        Self::name(value).parse().expect("a count within a u64")
    }

    fn list(value: &Value) -> impl Iterator<Item = &Value> {
        value.as_array().expect("a list").iter()
    }

    fn name(value: &Value) -> &str {
        value.as_str().expect("a string")
    }

    fn names(value: &Value) -> impl Iterator<Item = &str> {
        Self::list(value).map(Self::name)
    }

    fn row(value: &Value) -> Row {
        let next = Self::name(&value["next"]);
        Row {
            id: Self::string(&value["id"]),
            stages: Self::names(&value["stages"]).map(Stage::of).collect(),
            commands: Self::names(&value["commands"])
                .map(CommandName::of)
                .collect(),
            result: ResultName::of(Self::name(&value["result"])),
            outcomes: Self::names(&value["outcomes"])
                .map(OutcomeName::of)
                .collect(),
            next: (next != UNCHANGED).then(|| Stage::of(next)),
        }
    }

    fn string(value: &Value) -> String {
        String::from(Self::name(value))
    }
}
