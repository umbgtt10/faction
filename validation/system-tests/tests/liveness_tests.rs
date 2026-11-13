// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Liveness across every spawn and transport: members silenced or cut off by
// the faults the harness injects, every node suspecting at a threshold of
// two missed periods running.

use crate::support::log_dir;
use crate::support::real_time;
use faction::quorum::Quorum;
use faction::types::PeerId;
use faction_system_tests::cluster::Cluster;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::committer::Committer;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::transport_kind::TransportKind;
use rstest::rstest;
use std::path::PathBuf;

const COORDINATOR: usize = 1;
const NEWCOMER: PeerId = 3;
const SETTLE_ROUNDS: usize = 50;
const SUSPICION_THRESHOLD: usize = 2;

#[rstest]
#[case::task_real_inmemory(Spawn::Task, TransportKind::InMemory)]
#[case::task_real_channels(Spawn::Task, TransportKind::Channels)]
#[case::task_real_tcp(Spawn::Task, TransportKind::Tcp)]
#[case::task_real_grpc(Spawn::Task, TransportKind::Grpc)]
#[case::thread_real_inmemory(Spawn::Thread, TransportKind::InMemory)]
#[case::thread_real_channels(Spawn::Thread, TransportKind::Channels)]
#[case::thread_real_tcp(Spawn::Thread, TransportKind::Tcp)]
#[case::thread_real_grpc(Spawn::Thread, TransportKind::Grpc)]
#[case::process_real_tcp(Spawn::Process, TransportKind::Tcp)]
#[case::process_real_grpc(Spawn::Process, TransportKind::Grpc)]
fn a_member_cut_off_from_one_node_is_saved_by_indirect_probes(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = concluded("cut", 5, 3, spawn, transport);

    // Act
    cluster.cut(0, 4);
    cluster.settle(SETTLE_ROUNDS);

    // Assert
    assert!((0..5).all(|index| cluster.snapshot(index).suspected.is_empty()));
}

#[rstest]
#[case::task_real_inmemory(Spawn::Task, TransportKind::InMemory)]
#[case::task_real_channels(Spawn::Task, TransportKind::Channels)]
#[case::task_real_tcp(Spawn::Task, TransportKind::Tcp)]
#[case::task_real_grpc(Spawn::Task, TransportKind::Grpc)]
#[case::thread_real_inmemory(Spawn::Thread, TransportKind::InMemory)]
#[case::thread_real_channels(Spawn::Thread, TransportKind::Channels)]
#[case::thread_real_tcp(Spawn::Thread, TransportKind::Tcp)]
#[case::thread_real_grpc(Spawn::Thread, TransportKind::Grpc)]
#[case::process_real_tcp(Spawn::Process, TransportKind::Tcp)]
#[case::process_real_grpc(Spawn::Process, TransportKind::Grpc)]
fn a_member_heard_again_is_revived(#[case] spawn: Spawn, #[case] transport: TransportKind) {
    // Arrange
    let mut cluster = concluded("revived", 5, 3, spawn, transport);
    cluster.silence(4);
    cluster.poll_until(|cluster| suspected_by(cluster, &[0, 1, 2, 3], 4));

    // Act
    cluster.unsilence(4);
    cluster.poll_until(|cluster| (0..5).all(|index| cluster.snapshot(index).suspected.is_empty()));

    // Assert
    assert!((0..5).all(|index| cluster.snapshot(index).suspected.is_empty()));
}

#[rstest]
#[case::task_real_inmemory(Spawn::Task, TransportKind::InMemory)]
#[case::task_real_channels(Spawn::Task, TransportKind::Channels)]
#[case::task_real_tcp(Spawn::Task, TransportKind::Tcp)]
#[case::task_real_grpc(Spawn::Task, TransportKind::Grpc)]
#[case::thread_real_inmemory(Spawn::Thread, TransportKind::InMemory)]
#[case::thread_real_channels(Spawn::Thread, TransportKind::Channels)]
#[case::thread_real_tcp(Spawn::Thread, TransportKind::Tcp)]
#[case::thread_real_grpc(Spawn::Thread, TransportKind::Grpc)]
#[case::process_real_tcp(Spawn::Process, TransportKind::Tcp)]
#[case::process_real_grpc(Spawn::Process, TransportKind::Grpc)]
fn a_silenced_member_is_suspected_by_every_other(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = concluded("silenced", 5, 3, spawn, transport);

    // Act
    cluster.silence(4);
    cluster.poll_until(|cluster| suspected_by(cluster, &[0, 1, 2, 3], 4));

    // Assert
    assert!(suspected_by(&cluster, &[0, 1, 2, 3], 4));
    assert!((0..4).all(|index| cluster.snapshot(index).quorum == Quorum::Held));
}

#[rstest]
#[case::task_real_inmemory(Spawn::Task, TransportKind::InMemory)]
#[case::task_real_channels(Spawn::Task, TransportKind::Channels)]
#[case::task_real_tcp(Spawn::Task, TransportKind::Tcp)]
#[case::task_real_grpc(Spawn::Task, TransportKind::Grpc)]
#[case::thread_real_inmemory(Spawn::Thread, TransportKind::InMemory)]
#[case::thread_real_channels(Spawn::Thread, TransportKind::Channels)]
#[case::thread_real_tcp(Spawn::Thread, TransportKind::Tcp)]
#[case::thread_real_grpc(Spawn::Thread, TransportKind::Grpc)]
#[case::process_real_tcp(Spawn::Process, TransportKind::Tcp)]
#[case::process_real_grpc(Spawn::Process, TransportKind::Grpc)]
fn an_added_member_is_probed_once_committed(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = concluded("added", 3, 2, spawn, transport);
    cluster.add(
        COORDINATOR,
        NEWCOMER,
        Committer::Commit {
            required_count: None,
        },
    );
    cluster.poll_until_bootstrapped();

    // Act
    cluster.silence(NEWCOMER);
    cluster.poll_until(|cluster| suspected_by(cluster, &[0, 1, 2], NEWCOMER));

    // Assert
    assert!(suspected_by(&cluster, &[0, 1, 2], NEWCOMER));
}

fn concluded(
    scenario: &str,
    size: usize,
    required: usize,
    spawn: Spawn,
    transport: TransportKind,
) -> Cluster {
    let mut cluster = ClusterBuilder::new(size, required, real_time())
        .spawn(spawn)
        .transport(transport)
        .suspicion_threshold(SUSPICION_THRESHOLD)
        .log_dir(scenario_log(scenario, spawn, transport))
        .build();
    cluster.poll_until_bootstrapped();
    cluster
}

fn scenario_log(scenario: &str, spawn: Spawn, transport: TransportKind) -> PathBuf {
    log_dir(&format!("liveness-{scenario}-{spawn:?}-{transport:?}"))
}

#[rstest]
#[case::task_real_inmemory(Spawn::Task, TransportKind::InMemory)]
#[case::task_real_channels(Spawn::Task, TransportKind::Channels)]
#[case::task_real_tcp(Spawn::Task, TransportKind::Tcp)]
#[case::task_real_grpc(Spawn::Task, TransportKind::Grpc)]
#[case::thread_real_inmemory(Spawn::Thread, TransportKind::InMemory)]
#[case::thread_real_channels(Spawn::Thread, TransportKind::Channels)]
#[case::thread_real_tcp(Spawn::Thread, TransportKind::Tcp)]
#[case::thread_real_grpc(Spawn::Thread, TransportKind::Grpc)]
#[case::process_real_tcp(Spawn::Process, TransportKind::Tcp)]
#[case::process_real_grpc(Spawn::Process, TransportKind::Grpc)]
fn silencing_too_many_loses_quorum_and_their_return_restores_it(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = concluded("quorum", 5, 3, spawn, transport);
    for peer_id in 2..5 {
        cluster.silence(peer_id);
    }
    cluster
        .poll_until(|cluster| (0..2).all(|index| cluster.snapshot(index).quorum == Quorum::Lost));

    // Act
    for peer_id in 2..5 {
        cluster.unsilence(peer_id);
    }
    cluster
        .poll_until(|cluster| (0..5).all(|index| cluster.snapshot(index).quorum == Quorum::Held));

    // Assert
    assert!((0..5).all(|index| cluster.snapshot(index).quorum == Quorum::Held));
}

// Whether every node of `nodes` suspects `peer_id`.
fn suspected_by(cluster: &Cluster, nodes: &[usize], peer_id: PeerId) -> bool {
    nodes
        .iter()
        .all(|index| cluster.snapshot(*index).suspected.contains(&peer_id))
}
