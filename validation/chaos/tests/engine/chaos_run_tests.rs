// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::draw::generator::Generator;
use faction_chaos::engine::chaos_engine::ChaosEngine;
use faction_chaos::engine::chaos_run::ChaosRun;
use faction_chaos::engine::deterministic_engine::DeterministicEngine;
use faction_chaos::engine::real_engine::RealEngine;
use faction_chaos::engine::regressions::Regressions;
use faction_chaos::model::change::Change;
use faction_chaos::model::planned_change::PlannedChange;
use faction_chaos::model::scenario::Scenario;
use faction_chaos::oracle::failure::Failure;
use faction_system_tests::infra::timer::real::std_monotonic_clock::StdMonotonicClock;
use faction_system_tests::infra::timer::real::system_wall_clock::SystemWallClock;
use faction_system_tests::infra::timer::real::thread_pause::ThreadPause;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::time_ports::TimePorts;
use faction_system_tests::transport_kind::TransportKind;
use rstest::rstest;
use std::env::temp_dir;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::path::PathBuf;
use std::process::id;
use std::sync::Arc;

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
        changes_in_flight: Vec::new(),
        changes,
        faults: Vec::new(),
        restarts: Vec::new(),
        delivery_lag: 0,
        duplicates: 0,
        chaos_rounds,
        quiescence_rounds,
    }
}

fn deterministic() -> Box<dyn ChaosEngine> {
    Box::new(DeterministicEngine::new())
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
            ChaosRun::new(Generator::new(seed).scenario(), deterministic(), None)
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
        deterministic(),
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
        deterministic(),
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
        deterministic(),
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
    let run = ChaosRun::new(calm(3, Vec::new(), 0, 0), deterministic(), None);

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
fn run_of_each_regression_recorded_reaches_its_end_state() {
    // Arrange
    let recorded =
        Regressions::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("chaos-regressions.json"))
            .load()
            .unwrap();

    // Act
    let failures: Vec<String> = recorded
        .into_iter()
        .filter_map(|regression| {
            ChaosRun::new(regression.scenario, deterministic(), None)
                .run()
                .err()
                .map(|failure| format!("seed {}: {failure}", regression.seed))
        })
        .collect();

    // Assert
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn run_of_one_seed_twice_ends_alike() {
    // Arrange
    let first = ChaosRun::new(Generator::new(5).scenario(), deterministic(), None).run();

    // Act
    let second = ChaosRun::new(Generator::new(5).scenario(), deterministic(), None).run();

    // Assert
    assert_eq!(first, second);
}

#[test]
fn run_of_one_seed_twice_writes_the_same_logs() {
    // Arrange
    let (first, second) = (folder("first"), folder("second"));
    ChaosRun::new(
        Generator::new(11).scenario(),
        deterministic(),
        Some(first.clone()),
    )
    .run()
    .ok();

    // Act
    ChaosRun::new(
        Generator::new(11).scenario(),
        deterministic(),
        Some(second.clone()),
    )
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

#[test]
fn run_on_the_real_engine_of_a_calm_scenario_reaches_its_end_state() {
    // Arrange
    let time = TimePorts {
        clock: Arc::new(StdMonotonicClock::new()),
        pause: Arc::new(ThreadPause),
        wall_clock: Arc::new(SystemWallClock),
    };
    let run = ChaosRun::new(
        calm(
            2,
            vec![planned(Change::Addition { peer_id: 3 }, 0)],
            20,
            1_000,
        ),
        Box::new(RealEngine::new(
            Spawn::Thread,
            TransportKind::InMemory,
            time,
        )),
        None,
    );

    // Act
    let ended = run.run();

    // Assert
    assert!(ended.is_ok(), "{ended:?}");
}
