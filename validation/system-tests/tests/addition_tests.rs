// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::log_dir;
use crate::support::real_time;
use faction::peer_state::PeerState;
use faction::reconfiguration::Reconfiguration;
use faction_system_tests::cluster::Cluster;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::committer::Committer;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::transport_kind::TransportKind;
use rstest::rstest;
use std::path::PathBuf;

const COORDINATOR: usize = 1;
const SETTLE_ROUNDS: usize = 50;

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
fn a_committed_addition_raises_the_required_count(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = concluded("raises", spawn, transport);

    // Act
    cluster.add(
        COORDINATOR,
        3,
        Committer::Commit {
            required_count: Some(3),
        },
    );
    cluster.poll_until_bootstrapped();

    // Assert
    assert!(cluster.is_bootstrapped());
    assert!((0..4).all(|index| cluster.snapshot(index).required_count == 3));
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
fn a_second_addition_is_refused_while_one_is_in_flight(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = concluded("second", spawn, transport);
    cluster.add_newcomer(3);
    cluster.add_newcomer(4);

    // Act
    cluster.request_addition(COORDINATOR, 3);
    cluster.request_addition(COORDINATOR, 4);
    let in_flight = cluster.snapshot(COORDINATOR).reconfiguration;
    cluster.poll_until_proposed(COORDINATOR, 3);
    cluster.commit_addition(3, None);
    cluster.request_addition(COORDINATOR, 4);
    cluster.poll_until_proposed(COORDINATOR, 4);
    cluster.commit_addition(4, None);
    cluster.poll_until_bootstrapped();

    // Assert
    assert!(matches!(
        in_flight,
        Reconfiguration::AdditionRequested { peer_id: 3 }
            | Reconfiguration::AdditionAcknowledged { peer_id: 3 }
    ));
    assert!(cluster.is_bootstrapped());
    assert!((0..5).all(|index| cluster.member_count(index) == 5));
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
fn additions_one_after_another_each_converge(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = concluded("in-turn", spawn, transport);

    // Act
    cluster.add(
        COORDINATOR,
        3,
        Committer::Commit {
            required_count: None,
        },
    );
    cluster.poll_until_bootstrapped();
    cluster.add(
        2,
        4,
        Committer::Commit {
            required_count: None,
        },
    );
    cluster.poll_until_bootstrapped();

    // Assert
    assert!(cluster.is_bootstrapped());
    assert!((0..5).all(|index| cluster.member_count(index) == 5));
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
fn an_aborted_addition_never_counts_the_newcomer(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = concluded("aborted", spawn, transport);

    // Act
    cluster.add(COORDINATOR, 3, Committer::Abort);
    cluster.settle(SETTLE_ROUNDS);

    // Assert
    assert!((0..3).all(|index| cluster.member_count(index) == 3));
    assert_eq!(
        cluster.snapshot(COORDINATOR).reconfiguration,
        Reconfiguration::None
    );
    assert_ne!(cluster.node_state(3), PeerState::Bootstrapped);
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
fn an_added_newcomer_is_committed_and_bootstraps(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
) {
    // Arrange
    let mut cluster = concluded("committed", spawn, transport);

    // Act
    cluster.add(
        COORDINATOR,
        3,
        Committer::Commit {
            required_count: None,
        },
    );
    cluster.poll_until_bootstrapped();

    // Assert
    assert!(cluster.is_bootstrapped());
    assert_eq!(cluster.node_count(), 4);
    assert!((0..4).all(|index| cluster.member_count(index) == 4));
}

// Three nodes requiring two, concluded: the cluster an addition grows.
fn concluded(scenario: &str, spawn: Spawn, transport: TransportKind) -> Cluster {
    let mut cluster = ClusterBuilder::new(3, 2, real_time())
        .spawn(spawn)
        .transport(transport)
        .log_dir(scenario_log(scenario, spawn, transport))
        .build();
    cluster.poll_until_bootstrapped();
    cluster
}

fn scenario_log(scenario: &str, spawn: Spawn, transport: TransportKind) -> PathBuf {
    log_dir(&format!("add-{scenario}-{spawn:?}-{transport:?}"))
}
