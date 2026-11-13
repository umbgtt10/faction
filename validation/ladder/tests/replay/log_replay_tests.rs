// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The replay tier, one node: a node's log -- its configuration, then every call
// it reported, as the specification writes a step -- is valid against the
// schema, and replays whole into a node created from the configuration; a log
// altered, cut, reordered or opened with another configuration does not.

use crate::support::log_replay::LogReplay;
use crate::support::random_commands::RandomCommands;
use crate::support::spec_files::SpecFiles;
use crate::support::spec_log::Lines;
use crate::support::spec_log::SpecLog;
use crate::support::spec_schema::SpecSchema;
use crate::support::spec_writer::SpecWriter;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::quorum_policy::QuorumPolicy;
use proptest::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

fn config(required_count: usize) -> Config {
    Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(required_count), 1)
}

// The log a node writes of the given commands: its configuration, then a step
// per call.
fn log_of(config: Config, commands: &[Command]) -> Vec<Value> {
    let lines: Lines = Rc::new(RefCell::new(Vec::new()));
    let configuration = SpecWriter::configuration(&config);
    let mut faction = Faction::new(config, Box::new(SpecLog::new(Rc::clone(&lines))));
    for command in commands {
        let _ = faction.process(*command);
    }
    let steps = lines.borrow().clone();
    [vec![configuration], steps].concat()
}

fn pinging_log() -> Vec<Value> {
    log_of(
        config(4),
        &[
            Command::ParticipationObserved { peer_id: 1 },
            Command::ParticipationObserved { peer_id: 2 },
            Command::LocalParticipationCompleted,
            Command::ReadyObserved { peer_id: 3 },
        ],
    )
}

proptest! {
    #[test]
    fn admits_of_every_line_of_a_node_s_log_holds(
        required_count in 0usize..=6,
        commands in RandomCommands::sequence(40),
    ) {
        // Arrange
        let schema = SpecSchema::new(SpecFiles::schema());
        let log = log_of(config(required_count), &commands);

        // Act
        let configuration_admitted = schema.admits_configuration(&log[0]);
        let refused: Vec<&Value> = log[1..].iter().filter(|step| !schema.admits_step(step)).collect();

        // Assert
        prop_assert!(configuration_admitted, "{}", log[0]);
        prop_assert!(refused.is_empty(), "{refused:?}");
    }

    #[test]
    fn replay_of_a_node_s_own_log_reconstructs_every_line(
        required_count in 0usize..=6,
        commands in RandomCommands::sequence(40),
    ) {
        // Arrange
        let log = log_of(config(required_count), &commands);

        // Act
        let report = LogReplay::replay(&log);

        // Assert
        prop_assert!(report.departures.is_empty(), "{:?}", report.departures);
        prop_assert_eq!(report.lines, commands.len());
        prop_assert_eq!(report.reconstructed, commands.len());
    }
}

#[test]
fn replay_of_a_log_altered_at_a_line_departs_there() {
    // Arrange
    let mut log = pinging_log();
    log[3]["result"]["Accepted"]["outcomes"] = json!([]);

    // Act
    let report = LogReplay::replay(&log);

    // Assert
    assert_eq!(report.reconstructed, 3);
    assert_eq!(report.departures.len(), 1);
    assert!(
        report.departures[0].starts_with("line 4:"),
        "{:?}",
        report.departures
    );
}

#[test]
fn replay_of_a_log_cut_of_an_accepted_step_departs() {
    // Arrange
    let mut log = pinging_log();
    log.remove(3);

    // Act
    let report = LogReplay::replay(&log);

    // Assert
    assert!(!report.departures.is_empty());
    assert!(
        report.departures[0].starts_with("line 4:"),
        "{:?}",
        report.departures
    );
}

#[test]
fn replay_of_a_log_opened_with_another_configuration_departs() {
    // Arrange
    let mut log = pinging_log();
    log[0]["required_count"] = json!("2");

    // Act
    let report = LogReplay::replay(&log);

    // Assert
    assert!(!report.departures.is_empty());
    assert!(
        report.departures[0].starts_with("line 2:"),
        "{:?}",
        report.departures
    );
}

#[test]
fn replay_of_a_reordered_log_departs() {
    // Arrange
    let mut log = pinging_log();
    log.swap(1, 2);

    // Act
    let report = LogReplay::replay(&log);

    // Assert
    assert!(!report.departures.is_empty());
    assert!(
        report.departures[0].starts_with("line 2:"),
        "{:?}",
        report.departures
    );
}
