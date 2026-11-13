// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The conformance tier, the specification's model: every vector run against the
// independent model -- each step's result, then the final answer a closing Probe
// is given -- each vector deciding by exactly the rows it names and the vectors
// by every row of the matrix, the model knowing exactly the matrix's rows, and
// the matrix admitting what the prose states (§5). The vectors are checked here
// before the crate runs them.

use crate::support::spec_files::SpecFiles;
use faction_spec_model::answer::Answer;
use faction_spec_model::model::Model;
use faction_spec_model::rules::rule_book::RuleBook;
use faction_spec_model::spec_reader::SpecReader;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::stage::Stage;
use serde_json::json;
use serde_json::Value;
use std::collections::BTreeSet;

// The model's answers to a vector's steps, then to the closing Probe.
fn answers(vector: &Value) -> (Vec<Answer>, Answer) {
    let mut model = Model::new(
        SpecReader::table(&SpecFiles::matrix()),
        SpecReader::configuration(&vector["configuration"]),
    );
    let steps: Vec<Answer> = vector["steps"]
        .as_array()
        .expect("a vector's steps")
        .iter()
        .map(|step| model.process(&step["command"]))
        .collect();
    let closing = model.process(&json!({"Probe": {}}));
    (steps, closing)
}

fn matrix_row_ids() -> BTreeSet<String> {
    SpecFiles::matrix()["rows"]
        .as_array()
        .expect("the matrix's rows")
        .iter()
        .map(|row| String::from(row["id"].as_str().expect("a row's id")))
        .collect()
}

#[test]
fn admits_of_the_matrix_is_what_the_prose_states() {
    // Arrange
    let table = SpecReader::table(&SpecFiles::matrix());
    let every: Vec<CommandName> = CommandName::ALL.to_vec();
    let but = |refused: &[CommandName]| -> Vec<CommandName> {
        every
            .iter()
            .copied()
            .filter(|name| !refused.contains(name))
            .collect()
    };
    let addition = [
        CommandName::AdditionRequested,
        CommandName::MembershipAcknowledged,
        CommandName::AdditionAborted,
    ];
    let prose = [
        (
            Stage::Initial,
            but(&[[CommandName::DeadlineExpired].as_slice(), &addition].concat()),
        ),
        (Stage::Pinging, but(&addition)),
        (
            Stage::Collecting,
            but(&[
                [CommandName::LocalParticipationCompleted].as_slice(),
                &addition,
            ]
            .concat()),
        ),
        (
            Stage::Bootstrapped,
            vec![
                CommandName::ParticipationObserved,
                CommandName::JoinRequested,
                CommandName::JoinApproved,
                CommandName::JoinRejected,
                CommandName::AdditionRequested,
                CommandName::AdditionCommitted,
                CommandName::RequiredCountChanged,
                CommandName::Probe,
            ],
        ),
        (
            Stage::Reconfiguring,
            vec![
                CommandName::ParticipationObserved,
                CommandName::JoinRequested,
                CommandName::JoinRejected,
                CommandName::MembershipAcknowledged,
                CommandName::AdditionCommitted,
                CommandName::AdditionAborted,
                CommandName::RequiredCountChanged,
                CommandName::Probe,
            ],
        ),
    ];

    // Act
    let admitted: Vec<(Stage, Vec<CommandName>)> = Stage::ALL
        .iter()
        .map(|stage| (*stage, table.admits(*stage).to_vec()))
        .collect();

    // Assert
    assert_eq!(admitted, prose);
}

#[test]
fn process_of_every_vector_answers_each_step_as_written() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let departures: Vec<String> = vectors
        .iter()
        .flat_map(|(name, vector)| {
            let (steps, _) = answers(vector);
            steps
                .into_iter()
                .zip(vector["steps"].as_array().expect("a vector's steps"))
                .enumerate()
                .filter(|(_, (answer, step))| answer.result != step["result"])
                .map(|(index, (answer, step))| {
                    format!(
                        "{name}, step {}: the model answers {}, the vector {}",
                        index + 1,
                        answer.result,
                        step["result"]
                    )
                })
                .collect::<Vec<String>>()
        })
        .collect();

    // Assert
    assert_eq!(vectors.len(), 62);
    assert!(departures.is_empty(), "{departures:#?}");
}

#[test]
fn process_of_every_vector_answers_the_closing_probe_with_the_final_answer() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let departures: Vec<String> = vectors
        .iter()
        .filter_map(|(name, vector)| {
            let (_, closing) = answers(vector);
            let expected = json!({ "Probed": vector["final"] });
            (closing.result != expected).then(|| {
                format!(
                    "{name}: the model answers {}, the vector {expected}",
                    closing.result
                )
            })
        })
        .collect();

    // Assert
    assert!(departures.is_empty(), "{departures:#?}");
}

#[test]
fn process_of_every_vector_decides_by_exactly_the_rows_it_names() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let departures: Vec<String> = vectors
        .iter()
        .filter_map(|(name, vector)| {
            let (steps, _) = answers(vector);
            let decided: BTreeSet<String> = steps.into_iter().map(|answer| answer.row).collect();
            let named: BTreeSet<String> = vector["rows"]
                .as_array()
                .expect("a vector's rows")
                .iter()
                .map(|row| String::from(row.as_str().expect("a row's id")))
                .collect();
            (decided != named).then(|| format!("{name}: names {named:?}, decides by {decided:?}"))
        })
        .collect();

    // Assert
    assert!(departures.is_empty(), "{departures:#?}");
}

#[test]
fn process_of_the_vectors_decides_by_every_row_of_the_matrix() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let decided: BTreeSet<String> = vectors
        .iter()
        .flat_map(|(_, vector)| answers(vector).0)
        .map(|answer| answer.row)
        .collect();

    // Assert
    assert_eq!(decided, matrix_row_ids());
}

#[test]
fn row_ids_of_the_model_are_the_rows_of_the_matrix() {
    // Arrange & Act
    let known: BTreeSet<String> = RuleBook::row_ids().into_iter().map(String::from).collect();

    // Assert
    assert_eq!(known, matrix_row_ids());
}
