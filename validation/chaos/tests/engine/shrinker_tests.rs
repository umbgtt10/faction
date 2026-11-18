// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::engine::shrinker::Shrinker;
use faction_chaos::engine::shrunk::Shrunk;
use faction_chaos::model::change::Change;
use faction_chaos::model::fault::Fault;
use faction_chaos::model::fault_window::FaultWindow;
use faction_chaos::model::planned_change::PlannedChange;
use faction_chaos::model::restart_from::RestartFrom;
use faction_chaos::model::restart_plan::RestartPlan;
use faction_chaos::model::scenario::Scenario;
use faction_chaos::oracle::failure::Failure;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;

// Three nodes, two required, under chaos: a member removed and taken back,
// a lossy link, a crash, late and doubled commits -- and the removal of a
// peer that is no member, which never commits.
fn failing() -> Scenario {
    Scenario {
        seed: 0,
        nodes: 3,
        required_count: 2,
        suspicion_threshold: 1,
        changes_in_flight: Vec::new(),
        changes: vec![
            planned(Change::Removal { peer_id: 2 }, None),
            planned(Change::Addition { peer_id: 2 }, Some(0)),
            planned(Change::Removal { peer_id: 9 }, None),
        ],
        faults: vec![FaultWindow {
            peer_id: 1,
            from: 5,
            until: 40,
            fault: Fault::Transport(FaultPolicy {
                loss: 30,
                ..FaultPolicy::default()
            }),
        }],
        restarts: vec![RestartPlan {
            peer_id: 0,
            at: 10,
            down_for: 5,
            from: RestartFrom::Genesis,
        }],
        delivery_lag: 3,
        duplicates: 10,
        chaos_rounds: 64,
        quiescence_rounds: 300,
    }
}

fn planned(change: Change, after: Option<usize>) -> PlannedChange {
    PlannedChange {
        change,
        due: 0,
        after,
    }
}

#[test]
fn shrink_of_a_failing_scenario_keeps_only_what_fails() {
    // Arrange
    let shrinker = Shrinker::new(failing());

    // Act
    let shrunk = shrinker.shrink();

    // Assert
    assert_eq!(
        shrunk,
        Some(Shrunk {
            scenario: Scenario {
                changes: vec![planned(Change::Removal { peer_id: 9 }, None)],
                faults: Vec::new(),
                restarts: Vec::new(),
                delivery_lag: 0,
                duplicates: 0,
                chaos_rounds: 0,
                ..failing()
            },
            failure: Failure::Uncommitted { round: 300 },
        })
    );
}

#[test]
fn shrink_of_a_scenario_that_passes_is_nothing() {
    // Arrange
    let shrinker = Shrinker::new(Scenario {
        changes: Vec::new(),
        ..failing()
    });

    // Act
    let shrunk = shrinker.shrink();

    // Assert
    assert_eq!(shrunk, None);
}
