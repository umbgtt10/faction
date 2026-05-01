// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::transport_kind::TransportKind;
use rstest::rstest;

#[rstest]
#[case::task_in_memory(Spawn::Task, TransportKind::InMemory)]
fn cluster_reaches_bootstrapped(#[case] spawn: Spawn, #[case] transport: TransportKind) {
    let mut cluster = ClusterBuilder::new(5, 4)
        .spawn(spawn)
        .transport(transport)
        .build();

    cluster.poll_until_bootstrapped(10);
    assert!(cluster.is_bootstrapped());
}
