// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::spec_files::SpecFiles;
use faction_spec_model::model::Model;
use faction_spec_model::spec_reader::SpecReader;
use serde_json::json;

// Node 1 of members 1, 2 and 3, requiring two.
fn model() -> Model {
    Model::new(
        SpecReader::table(&SpecFiles::matrix()),
        SpecReader::configuration(
            &json!({"peer_id": "1", "peers": ["1", "2", "3"], "required_count": "2"}),
        ),
    )
}

#[test]
fn process_of_a_deadline_on_a_fresh_node_is_rejected_by_not_admitted() {
    // Arrange
    let mut model = model();

    // Act
    let answer = model.process(&json!({"DeadlineExpired": {}}));

    // Assert
    assert_eq!(answer.row, "not-admitted");
    assert_eq!(
        answer.result["Rejected"]["cluster_view"]["peer_state"],
        json!({"Fresh": {}})
    );
}

#[test]
fn process_of_a_probe_on_a_fresh_node_is_probed_with_its_view() {
    // Arrange
    let mut model = model();

    // Act
    let answer = model.process(&json!({"Probe": {}}));

    // Assert
    assert_eq!(answer.row, "probed");
    assert_eq!(
        answer.result,
        json!({"Probed": {
            "cluster_view": {
                "peer_state": {"Fresh": {}},
                "is_pinging_completed": false,
                "pinging_peers": [],
                "collecting_peers": [],
                "required_count": "2",
                "deadline_missed": false,
                "members": ["1", "2", "3"],
                "addition": {"None": {}},
            },
            "admissible": [
                "ParticipationObserved",
                "ReadyObserved",
                "LocalParticipationCompleted",
                "JoinRequested",
                "JoinApproved",
                "JoinRejected",
                "AdditionCommitted",
                "RequiredCountChanged",
                "Probe",
            ],
        }})
    );
}

#[test]
fn process_of_an_addition_requested_of_a_concluded_node_reconfigures_it_still_bootstrapped() {
    // Arrange
    let mut model = model();
    let _ = model.process(&json!({"LocalParticipationCompleted": {}}));
    let _ = model.process(&json!({"ReadyObserved": {"peer_id": "2"}}));

    // Act
    let answer = model.process(&json!({"AdditionRequested": {"peer_id": "4"}}));

    // Assert
    assert_eq!(answer.row, "addition-requested");
    assert_eq!(
        answer.result["Accepted"]["outcomes"],
        json!([{"SendMembership": {"peer_id": "4"}}])
    );
    assert_eq!(
        answer.result["Accepted"]["cluster_view"]["peer_state"],
        json!({"Bootstrapped": {}})
    );
    assert_eq!(
        answer.result["Accepted"]["cluster_view"]["addition"],
        json!({"Requested": {"peer_id": "4"}})
    );
    assert_eq!(
        answer.result["Accepted"]["admissible"],
        json!([
            "ParticipationObserved",
            "JoinRequested",
            "JoinRejected",
            "MembershipAcknowledged",
            "AdditionCommitted",
            "AdditionAborted",
            "RequiredCountChanged",
            "Probe",
        ])
    );
}

#[test]
fn process_of_the_readiness_the_count_lacks_concludes() {
    // Arrange
    let mut model = model();
    let _ = model.process(&json!({"LocalParticipationCompleted": {}}));

    // Act
    let answer = model.process(&json!({"ReadyObserved": {"peer_id": "3"}}));

    // Assert
    assert_eq!(answer.row, "ready-concluded");
    assert_eq!(
        answer.result["Accepted"]["outcomes"],
        json!([
            {"ReadyAccepted": {"peer_id": "3"}},
            {"Concluded": {"mode": {"Bootstrapped": {}}}},
        ])
    );
    assert_eq!(
        answer.result["Accepted"]["cluster_view"]["collecting_peers"],
        json!(["1", "3"])
    );
    assert_eq!(
        answer.result["Accepted"]["admissible"],
        json!([
            "ParticipationObserved",
            "JoinRequested",
            "JoinApproved",
            "JoinRejected",
            "AdditionRequested",
            "AdditionCommitted",
            "RequiredCountChanged",
            "Probe",
        ])
    );
}
