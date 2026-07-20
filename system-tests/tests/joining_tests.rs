// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use chrono::Utc;
use faction::peer_state::PeerState;
use faction_system_tests::approver::Approver;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::transport_kind::TransportKind;
use rstest::rstest;

const SETTLE_ROUNDS: usize = 50;

fn log_path(scenario: &str, spawn: Spawn, transport: TransportKind) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let name = format!("{timestamp}_join_{scenario}_{spawn:?}_{transport:?}.jsonl").to_lowercase();
    PathBuf::from("logs").join(name)
}

#[rstest]
#[case::task_inmemory(Spawn::Task, TransportKind::InMemory)]
fn cold_newcomer_joins_a_bootstrapped_cluster_and_converges(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 2)
        .spawn(spawn)
        .transport(transport)
        .log_path(log_path("converge", spawn, transport))
        .build();
    cluster.poll_until_bootstrapped();

    // Act
    cluster.join(3, Approver::AcceptAll);
    cluster.poll_until_bootstrapped();

    // Assert
    assert!(cluster.is_bootstrapped());
    assert_eq!(cluster.node_count(), 4);
}

#[rstest]
#[case::task_inmemory(Spawn::Task, TransportKind::InMemory)]
fn a_rejected_newcomer_is_denied_and_never_counts(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 2)
        .spawn(spawn)
        .transport(transport)
        .log_path(log_path("rejected", spawn, transport))
        .build();
    cluster.poll_until_bootstrapped();

    // Act
    cluster.join(3, Approver::RejectAll);
    cluster.settle(SETTLE_ROUNDS);

    // Assert
    assert_eq!(cluster.node_state(0), PeerState::Bootstrapped);
    assert_eq!(cluster.node_state(1), PeerState::Bootstrapped);
    assert_eq!(cluster.node_state(2), PeerState::Bootstrapped);
    assert_ne!(cluster.node_state(3), PeerState::Bootstrapped);
    assert_eq!(cluster.member_count(0), 3);
}

#[rstest]
#[case::task_inmemory(Spawn::Task, TransportKind::InMemory)]
fn a_duplicate_join_is_ignored_and_membership_is_stable(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 2)
        .spawn(spawn)
        .transport(transport)
        .log_path(log_path("duplicate", spawn, transport))
        .build();
    cluster.poll_until_bootstrapped();

    // Act
    cluster.join(3, Approver::AcceptAll);
    let after_first_admission = cluster.member_count(0);
    cluster.admit(3);
    let after_duplicate = cluster.member_count(0);
    cluster.poll_until_bootstrapped();

    // Assert
    assert_eq!(after_first_admission, 4);
    assert_eq!(after_duplicate, 4);
    assert!(cluster.is_bootstrapped());
}

#[rstest]
#[case::task_inmemory(Spawn::Task, TransportKind::InMemory)]
fn concurrent_newcomers_each_join_and_converge(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 2)
        .spawn(spawn)
        .transport(transport)
        .log_path(log_path("concurrent", spawn, transport))
        .build();
    cluster.poll_until_bootstrapped();

    // Act
    cluster.join(3, Approver::AcceptAll);
    cluster.join(4, Approver::AcceptAll);
    cluster.poll_until_bootstrapped();

    // Assert
    assert!(cluster.is_bootstrapped());
    assert_eq!(cluster.node_count(), 5);
    assert_eq!(cluster.member_count(0), 5);
}
