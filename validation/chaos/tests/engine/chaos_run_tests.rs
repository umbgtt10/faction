// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::draw::generator::Generator;
use faction_chaos::engine::chaos_run::ChaosRun;
use faction_chaos::model::change::Change;
use faction_chaos::model::planned_change::PlannedChange;
use faction_chaos::model::scenario::Scenario;
use faction_chaos::oracle::failure::Failure;
use rstest::rstest;

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
            ChaosRun::new(Generator::new(seed).scenario())
                .run()
                .err()
                .map(|failure| format!("seed {seed}: {failure}"))
        })
        .collect();

    // Assert
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn run_of_a_calm_scenario_reaches_its_end_state() {
    // Arrange
    let run = ChaosRun::new(calm(
        2,
        vec![
            planned(Change::Addition { peer_id: 3 }, 0),
            planned(Change::Removal { peer_id: 1 }, 10),
        ],
        50,
        2_000,
    ));

    // Act
    let ended = run.run();

    // Assert
    assert!(ended.is_ok(), "{ended:?}");
}

#[test]
fn run_of_a_change_that_never_commits_fails_uncommitted() {
    // Arrange
    let run = ChaosRun::new(calm(
        3,
        vec![planned(Change::Removal { peer_id: 2 }, 0)],
        0,
        300,
    ));

    // Act
    let ended = run.run();

    // Assert
    assert_eq!(ended, Err(Failure::Uncommitted { round: 300 }));
}

#[test]
fn run_of_a_scenario_given_no_time_fails_diverged() {
    // Arrange
    let run = ChaosRun::new(calm(3, Vec::new(), 0, 0));

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
    let first = ChaosRun::new(Generator::new(5).scenario()).run();

    // Act
    let second = ChaosRun::new(Generator::new(5).scenario()).run();

    // Assert
    assert_eq!(first, second);
}
