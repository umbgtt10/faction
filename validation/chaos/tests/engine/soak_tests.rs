// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::draw::scenario_source::ScenarioSource;
use faction_chaos::engine::regressions::Regressions;
use faction_chaos::engine::soak::Soak;
use faction_chaos::engine::soak_limit::SoakLimit;
use faction_chaos::model::change::Change;
use faction_chaos::model::planned_change::PlannedChange;
use faction_chaos::model::scenario::Scenario;
use faction_system_tests::infra::timer::simulated::simulated_clock::SimulatedClock;
use std::env::temp_dir;
use std::fs::remove_file;
use std::io::Result as IoResult;
use std::io::Write;
use std::path::PathBuf;
use std::process::id;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

// Three calm nodes for every seed but 1, whose scenario removes a peer that
// is no member, which never commits.
struct OneFailing;

impl ScenarioSource for OneFailing {
    fn scenario(&self, seed: u64) -> Scenario {
        let changes = if seed == 1 {
            vec![PlannedChange {
                change: Change::Removal { peer_id: 9 },
                due: 0,
                after: None,
            }]
        } else {
            Vec::new()
        };
        Scenario {
            seed,
            nodes: 3,
            required_count: 2,
            suspicion_threshold: 1,
            changes,
            faults: Vec::new(),
            restarts: Vec::new(),
            delivery_lag: 0,
            duplicates: 0,
            chaos_rounds: 20,
            quiescence_rounds: 300,
        }
    }
}

#[derive(Clone, Default)]
struct SharedOutput {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl SharedOutput {
    fn text(&self) -> String {
        String::from_utf8(self.bytes.lock().unwrap().clone()).unwrap()
    }
}

impl Write for SharedOutput {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.bytes.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

fn soak(from: u64, limit: SoakLimit, path: PathBuf) -> (Soak, SharedOutput) {
    let output = SharedOutput::default();
    let soak = Soak::new(
        from,
        limit,
        Box::new(OneFailing),
        Regressions::new(path),
        Arc::new(SimulatedClock::at(Duration::ZERO)),
        Box::new(output.clone()),
    );
    (soak, output)
}

#[test]
fn run_of_a_soak_given_no_time_plays_nothing() {
    // Arrange
    let path = temp_dir().join(format!("soak-tests-no-time-{}.json", id()));
    let (soak, output) = soak(5, SoakLimit::For(Duration::ZERO), path);

    // Act
    let failed = soak.run();

    // Assert
    assert_eq!(failed, 0);
    assert_eq!(output.text(), "soak from seed 5: 0 scenarios, 0 failed\n");
}

#[test]
fn run_of_a_soak_meeting_a_failing_seed_records_it_shrunk() {
    // Arrange
    let path = temp_dir().join(format!("soak-tests-failing-{}.json", id()));
    let (soak, output) = soak(0, SoakLimit::Scenarios(3), path.clone());

    // Act
    let failed = soak.run();

    // Assert
    let recorded = Regressions::new(path.clone()).load().unwrap();
    remove_file(&path).ok();
    assert_eq!(failed, 1);
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].seed, 1);
    assert_eq!(recorded[0].scenario.chaos_rounds, 0);
    assert_eq!(
        output.text(),
        "seed 1 failed: round 320: a planned change never committed\n\
         seed 1 shrunk: round 300: a planned change never committed\n\
         soak from seed 0: 3 scenarios, 1 failed\n"
    );
}
