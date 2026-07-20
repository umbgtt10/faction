// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::approver::Approver;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::transport_kind::TransportKind;
use rstest::rstest;

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
        .build();
    cluster.poll_until_bootstrapped();

    // Act
    cluster.join(3, Approver::AcceptAll);
    cluster.poll_until_bootstrapped();

    // Assert
    assert!(cluster.is_bootstrapped());
}
