// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::reconfiguration::Reconfiguration;
use faction_chaos::oracle::failure::Failure;
use faction_system_tests::infra::invariants::breach::Breach;
use faction_system_tests::node_snapshot::NodeSnapshot;

#[test]
fn to_string_of_a_breach_names_the_node_the_invariant_and_the_command() {
    // Arrange
    let failure = Failure::Breach {
        round: 12,
        breach: Breach {
            peer_id: 2,
            invariant: "the epoch fell",
            command: Command::Probe,
        },
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(text, "round 12: node 2 broke \"the epoch fell\" on Probe");
}

#[test]
fn to_string_of_a_change_never_committed_says_so() {
    // Arrange
    let failure = Failure::Uncommitted { round: 7 };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(text, "round 7: a planned change never committed");
}

#[test]
fn to_string_of_a_divergence_names_what_was_expected_and_found() {
    // Arrange
    let failure = Failure::Diverged {
        round: 40,
        peer_id: 1,
        expected: String::from("Removed at epoch 2"),
        found: Box::new(NodeSnapshot {
            peer_state: PeerState::Bootstrapped,
            member_count: 1,
            members: vec![1],
            required_count: 1,
            reconfiguration: Reconfiguration::None,
            suspected: Vec::new(),
            quorum: Quorum::Held,
            epoch: 1,
        }),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert!(text.starts_with("round 40: node 1 expected Removed at epoch 2, found NodeSnapshot"));
}
