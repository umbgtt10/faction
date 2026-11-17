// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::draw::generator::Generator;
use faction_chaos::engine::regression::Regression;
use faction_chaos::engine::regressions::Regressions;
use faction_chaos::engine::regressions_error::RegressionsError;
use std::env::temp_dir;
use std::fs::remove_file;
use std::fs::write;
use std::path::PathBuf;
use std::process::id;

fn path(label: &str) -> PathBuf {
    temp_dir().join(format!("regressions-tests-{label}-{}.json", id()))
}

#[test]
fn load_of_a_file_not_yet_written_holds_none() {
    // Arrange
    let regressions = Regressions::new(path("missing"));

    // Act
    let loaded = regressions.load();

    // Assert
    assert_eq!(loaded, Ok(Vec::new()));
}

#[test]
fn load_of_a_file_that_is_no_list_of_regressions_names_it() {
    // Arrange
    let path = path("malformed");
    write(&path, "not json").unwrap();

    // Act
    let loaded = Regressions::new(path.clone()).load();

    // Assert
    remove_file(&path).ok();
    assert!(matches!(loaded, Err(RegressionsError { path: named, .. }) if named == path));
}

#[test]
fn record_adds_the_regression_whole_to_those_the_file_holds() {
    // Arrange
    let path = path("record");
    let regressions = Regressions::new(path.clone());
    let first = Regression {
        seed: 4,
        failure: String::from("round 9: a planned change never committed"),
        scenario: Generator::new(4).scenario(),
    };
    let second = Regression {
        seed: 23,
        failure: String::from("round 12: node 1 expected Removed at epoch 2"),
        scenario: Generator::new(23).scenario(),
    };
    regressions.record(first.clone()).unwrap();

    // Act
    regressions.record(second.clone()).unwrap();

    // Assert
    let loaded = regressions.load();
    remove_file(&path).ok();
    assert_eq!(loaded, Ok(vec![first, second]));
}

#[test]
fn record_of_a_seed_already_held_keeps_the_one_held() {
    // Arrange
    let path = path("record-twice");
    let regressions = Regressions::new(path.clone());
    let held = Regression {
        seed: 4,
        failure: String::from("round 9: a planned change never committed"),
        scenario: Generator::new(4).scenario(),
    };
    regressions.record(held.clone()).unwrap();

    // Act
    regressions
        .record(Regression {
            failure: String::from("round 11: a planned change never committed"),
            ..held.clone()
        })
        .unwrap();

    // Assert
    let loaded = regressions.load();
    remove_file(&path).ok();
    assert_eq!(loaded, Ok(vec![held]));
}

#[test]
fn to_string_of_an_error_names_the_file_and_the_reason() {
    // Arrange
    let error = RegressionsError {
        path: PathBuf::from("chaos-regressions.json"),
        reason: String::from("expected value at line 1 column 1"),
    };

    // Act
    let text = error.to_string();

    // Assert
    assert_eq!(
        text,
        "chaos-regressions.json: expected value at line 1 column 1"
    );
}
