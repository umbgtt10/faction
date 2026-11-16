// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::draw::generator::Generator;
use faction_chaos::engine::chaos_run::ChaosRun;
use faction_chaos::model::change::Change;
use faction_chaos::model::planned_change::PlannedChange;
use faction_chaos::model::scenario::Scenario;
use faction_chaos::oracle::failure::Failure;
use rstest::rstest;
use std::env::temp_dir;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::path::PathBuf;
use std::process::id;

const SEEDS_PER_CASE: u64 = 30;

fn calm(
    required_count: usize,
    changes: Vec<PlannedChange>,
    chaos_rounds: u64,
    quiescence_rounds: u64,
) -> Scenario {
    Scenario {
        seed: 0,
        nodes: 3,
        required_count,
        suspicion_threshold: 1,
        changes,
        faults: Vec::new(),
        restarts: Vec::new(),
        delivery_lag: 0,
        duplicates: 0,
        chaos_rounds,
        quiescence_rounds,
    }
}

fn folder(label: &str) -> PathBuf {
    temp_dir().join(format!("chaos-run-tests-{label}-{}", id()))
}

fn planned(change: Change, due: u64) -> PlannedChange {
    PlannedChange {
        change,
        due,
        after: None,
    }
}

// The single run's seeds: three hundred, in ten cases of thirty, each run on
// the deterministic engine to the end state its log implies.
#[rstest]
#[case(0)]
#[case(1)]
#[case(2)]
#[case(3)]
#[case(4)]
#[case(5)]
#[case(6)]
#[case(7)]
#[case(8)]
#[case(9)]
fn run_of_each_single_run_seed_reaches_its_end_state(#[case] case: u64) {
    // Arrange
    let seeds = case * SEEDS_PER_CASE..(case + 1) * SEEDS_PER_CASE;

    // Act
    let failures: Vec<String> = seeds
        .filter_map(|seed| {
            ChaosRun::new(Generator::new(seed).scenario(), None)
                .run()
                .err()
                .map(|failure| format!("seed {seed}: {failure}"))
        })
        .collect();

    // Assert
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn run_given_a_log_folder_writes_each_node_s_log_and_their_consolidation() {
    // Arrange
    let folder = folder("logs");
    let run = ChaosRun::new(
        calm(
            2,
            vec![planned(Change::Addition { peer_id: 3 }, 0)],
            50,
            2_000,
        ),
        Some(folder.clone()),
    );

    // Act
    let ended = run.run();

    // Assert
    let node = read_to_string(folder.join("node-3.jsonl")).unwrap();
    let consolidated = read_to_string(folder.join("consolidated.jsonl")).unwrap();
    remove_dir_all(&folder).ok();
    assert!(ended.is_ok(), "{ended:?}");
    assert!(node.contains("\"event\":\"start\""), "{node}");
    assert!(
        consolidated.starts_with("{\"timestamp\":\"2026-01-01T00:00:00"),
        "{consolidated}"
    );
}

#[test]
fn run_of_a_calm_scenario_reaches_its_end_state() {
    // Arrange
    let run = ChaosRun::new(
        calm(
            2,
            vec![
                planned(Change::Addition { peer_id: 3 }, 0),
                planned(Change::Removal { peer_id: 1 }, 10),
            ],
            50,
            2_000,
        ),
        None,
    );

    // Act
    let ended = run.run();

    // Assert
    assert!(ended.is_ok(), "{ended:?}");
}

#[test]
fn run_of_a_change_that_never_commits_fails_uncommitted() {
    // Arrange
    let run = ChaosRun::new(
        calm(3, vec![planned(Change::Removal { peer_id: 2 }, 0)], 0, 300),
        None,
    );

    // Act
    let ended = run.run();

    // Assert
    assert_eq!(ended, Err(Failure::Uncommitted { round: 300 }));
}

#[test]
fn run_of_a_scenario_given_no_time_fails_diverged() {
    // Arrange
    let run = ChaosRun::new(calm(3, Vec::new(), 0, 0), None);

    // Act
    let ended = run.run();

    // Assert
    assert!(
        matches!(
            ended,
            Err(Failure::Diverged {
                round: 0,
                peer_id: 0,
                ..
            })
        ),
        "{ended:?}"
    );
}

#[test]
fn run_of_one_seed_twice_ends_alike() {
    // Arrange
    let first = ChaosRun::new(Generator::new(5).scenario(), None).run();

    // Act
    let second = ChaosRun::new(Generator::new(5).scenario(), None).run();

    // Assert
    assert_eq!(first, second);
}

#[test]
fn run_of_one_seed_twice_writes_the_same_logs() {
    // Arrange
    let (first, second) = (folder("first"), folder("second"));
    ChaosRun::new(Generator::new(11).scenario(), Some(first.clone()))
        .run()
        .ok();

    // Act
    ChaosRun::new(Generator::new(11).scenario(), Some(second.clone()))
        .run()
        .ok();

    // Assert
    let logs =
        [&first, &second].map(|folder| read_to_string(folder.join("consolidated.jsonl")).unwrap());
    remove_dir_all(&first).ok();
    remove_dir_all(&second).ok();
    assert!(!logs[0].is_empty());
    assert_eq!(logs[0], logs[1]);
}
