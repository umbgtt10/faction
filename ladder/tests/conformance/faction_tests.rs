// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The conformance tier: every vector of the specification, run against the
// crate as the specification states it -- each step's result, then the final
// answer a closing Probe is given, with an observer that does nothing; every
// call reported, with the result answered, to one that keeps them -- every
// vector and the matrix valid against the schema, and every row of the matrix
// exercised.

use crate::support::recording_observer::RecordingObserver;
use crate::support::spec_files::SpecFiles;
use crate::support::spec_reader::SpecReader;
use crate::support::spec_schema::SpecSchema;
use crate::support::spec_writer::SpecWriter;
use faction::command::Command;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::process_result::ProcessResult;
use serde_json::json;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

// What a vector's run answered where the vector expected otherwise: nothing
// for a conforming implementation.
fn departures(name: &str, vector: &Value) -> Vec<String> {
    let mut faction = Faction::new(
        SpecReader::configuration(&vector["configuration"]),
        Box::new(NoOpObserver),
    );
    let mut departures = Vec::new();
    let steps = vector["steps"].as_array().expect("a vector's steps");
    for (index, step) in steps.iter().enumerate() {
        let result = SpecWriter::result(&faction.process(SpecReader::command(&step["command"])));
        if result != step["result"] {
            departures.push(format!(
                "{name}, step {}: answered {result}, not {}",
                index + 1,
                step["result"]
            ));
        }
    }
    let closing = SpecWriter::result(&faction.process(Command::Probe));
    let expected = json!({ "Probed": vector["final"] });
    if closing != expected {
        departures.push(format!(
            "{name}, final answer: answered {closing}, not {expected}"
        ));
    }
    departures
}

// What a vector's run reported to its observer where it answered otherwise:
// nothing for a conforming implementation, which reports each step's call and
// the closing Probe's, each with the result it answered, in order, and nothing
// else.
fn misreport(name: &str, vector: &Value) -> Option<String> {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut faction = Faction::new(
        SpecReader::configuration(&vector["configuration"]),
        Box::new(RecordingObserver::new(Rc::clone(&calls))),
    );
    let answered: Vec<(Command, ProcessResult)> = vector["steps"]
        .as_array()
        .expect("a vector's steps")
        .iter()
        .map(|step| SpecReader::command(&step["command"]))
        .chain([Command::Probe])
        .map(|command| (command, faction.process(command)))
        .collect();
    let reported = calls.borrow().clone();
    (reported != answered)
        .then(|| format!("{name}: reported {reported:?}, having answered {answered:?}"))
}

#[test]
fn admits_matrix_of_the_specification_holds() {
    // Arrange
    let schema = SpecSchema::new(SpecFiles::schema());

    // Act
    let admitted = schema.admits_matrix(&SpecFiles::matrix());

    // Assert
    assert!(
        admitted,
        "matrix.json is not valid against the schema's matrix"
    );
}

#[test]
fn admits_vector_of_every_vector_holds() {
    // Arrange
    let schema = SpecSchema::new(SpecFiles::schema());

    // Act
    let refused: Vec<String> = SpecFiles::vectors()
        .into_iter()
        .filter(|(_, vector)| !schema.admits_vector(vector))
        .map(|(name, _)| name)
        .collect();

    // Assert
    assert!(
        refused.is_empty(),
        "vectors not valid against the schema: {refused:?}"
    );
}

#[test]
fn process_of_every_vector_answers_its_results_and_its_final_answer() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let departures: Vec<String> = vectors
        .iter()
        .flat_map(|(name, vector)| departures(name, vector))
        .collect();

    // Assert
    assert!(!vectors.is_empty(), "no vector was read");
    assert!(departures.is_empty(), "{departures:#?}");
}

#[test]
fn process_of_every_vector_reports_every_call_to_the_observer_as_answered() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let misreports: Vec<String> = vectors
        .iter()
        .filter_map(|(name, vector)| misreport(name, vector))
        .collect();

    // Assert
    assert!(!vectors.is_empty(), "no vector was read");
    assert!(misreports.is_empty(), "{misreports:#?}");
}

#[test]
fn vectors_of_the_specification_exercise_every_row_of_the_matrix() {
    // Arrange
    let rows: BTreeSet<String> = SpecFiles::matrix()["rows"]
        .as_array()
        .expect("the matrix's rows")
        .iter()
        .map(|row| String::from(row["id"].as_str().expect("a row's id")))
        .collect();

    // Act
    let exercised: BTreeSet<String> = SpecFiles::vectors()
        .iter()
        .flat_map(|(_, vector)| {
            vector["rows"]
                .as_array()
                .expect("a vector's rows")
                .iter()
                .map(|row| String::from(row.as_str().expect("a row's id")))
                .collect::<Vec<String>>()
        })
        .collect();

    // Assert
    let unexercised: Vec<&String> = rows.difference(&exercised).collect();
    let unknown: Vec<&String> = exercised.difference(&rows).collect();
    assert!(
        unexercised.is_empty(),
        "rows no vector exercises: {unexercised:?}"
    );
    assert!(
        unknown.is_empty(),
        "rows the matrix does not have: {unknown:?}"
    );
}
