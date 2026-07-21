// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use faction::peer_state::PeerState;
use faction_system_tests::approver::Approver;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::transport_kind::TransportKind;
use rstest::rstest;

use crate::support::log_path;

const SETTLE_ROUNDS: usize = 50;

fn scenario_log(scenario: &str, spawn: Spawn, transport: TransportKind) -> PathBuf {
    log_path(&format!("join_{scenario}_{spawn:?}_{transport:?}"))
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
        .log_path(scenario_log("converge", spawn, transport))
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
        .log_path(scenario_log("rejected", spawn, transport))
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
        .log_path(scenario_log("duplicate", spawn, transport))
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
        .log_path(scenario_log("concurrent", spawn, transport))
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

#[rstest]
#[case::task_inmemory(Spawn::Task, TransportKind::InMemory)]
fn a_newcomer_admitted_before_bootstrap_still_converges(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 2)
        .spawn(spawn)
        .transport(transport)
        .log_path(scenario_log("before_bootstrap", spawn, transport))
        .build();

    // Act: admit the newcomer while the cluster is still converging, not yet bootstrapped
    cluster.start_all();
    cluster.join(3, Approver::AcceptAll);
    cluster.poll_until_bootstrapped();

    // Assert
    assert!(cluster.is_bootstrapped());
    assert_eq!(cluster.node_count(), 4);
    assert_eq!(cluster.member_count(0), 4);
}
