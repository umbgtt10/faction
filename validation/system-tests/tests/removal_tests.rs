// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Removal across every spawn and transport: the harness plays the layer that
// commits, node 0 coordinates, and every node suspects at a threshold of two
// missed periods running.

use crate::support::log_dir;
use crate::support::real_time;
use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::reconfiguration::Reconfiguration;
use faction_system_tests::cluster::Cluster;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::probe_stretch::ProbeStretch;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::transport_kind::TransportKind;
use rstest::rstest;
use std::path::PathBuf;

const COORDINATOR: usize = 0;
// Rounds a removed node is left silenced for, per unit of stretch: longer
// than a member silenced would take to be suspected.
const SETTLE_ROUNDS_PER_STRETCH: usize = 60;
const SUSPICION_THRESHOLD: usize = 2;

#[rstest]
#[case::task_real_inmemory(Spawn::Task, TransportKind::InMemory, ProbeStretch::Generous)]
#[case::task_real_channels(Spawn::Task, TransportKind::Channels, ProbeStretch::Generous)]
#[case::task_real_tcp(Spawn::Task, TransportKind::Tcp, ProbeStretch::Generous)]
#[case::task_real_grpc(Spawn::Task, TransportKind::Grpc, ProbeStretch::Generous)]
#[case::thread_real_inmemory(Spawn::Thread, TransportKind::InMemory, ProbeStretch::Minimal)]
#[case::thread_real_channels(Spawn::Thread, TransportKind::Channels, ProbeStretch::Minimal)]
#[case::thread_real_tcp(Spawn::Thread, TransportKind::Tcp, ProbeStretch::Minimal)]
#[case::thread_real_grpc(Spawn::Thread, TransportKind::Grpc, ProbeStretch::Minimal)]
#[case::process_real_tcp(Spawn::Process, TransportKind::Tcp, ProbeStretch::Minimal)]
#[case::process_real_grpc(Spawn::Process, TransportKind::Grpc, ProbeStretch::Minimal)]
fn a_committed_removal_lowers_the_required_count(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] probe_stretch: ProbeStretch,
) {
    // Arrange
    let mut cluster = concluded("lowered", 3, 3, spawn, transport, probe_stretch);
    cluster.silence(2);
    cluster.poll_until(|cluster| cluster.snapshot(COORDINATOR).suspected.contains(&2));
    cluster.request_removal(COORDINATOR, 2);
    cluster.poll_until_removal_proposed(COORDINATOR, 2);

    // Act
    cluster.commit_removal(2, Some(2));

    // Assert
    for index in 0..2 {
        let snapshot = cluster.snapshot(index);
        assert_eq!((snapshot.member_count, snapshot.required_count), (2, 2));
        assert_eq!(snapshot.quorum, Quorum::Held);
    }
}

#[rstest]
#[case::task_real_inmemory(Spawn::Task, TransportKind::InMemory, ProbeStretch::Generous)]
#[case::task_real_channels(Spawn::Task, TransportKind::Channels, ProbeStretch::Generous)]
#[case::task_real_tcp(Spawn::Task, TransportKind::Tcp, ProbeStretch::Generous)]
#[case::task_real_grpc(Spawn::Task, TransportKind::Grpc, ProbeStretch::Generous)]
#[case::thread_real_inmemory(Spawn::Thread, TransportKind::InMemory, ProbeStretch::Minimal)]
#[case::thread_real_channels(Spawn::Thread, TransportKind::Channels, ProbeStretch::Minimal)]
#[case::thread_real_tcp(Spawn::Thread, TransportKind::Tcp, ProbeStretch::Minimal)]
#[case::thread_real_grpc(Spawn::Thread, TransportKind::Grpc, ProbeStretch::Minimal)]
#[case::process_real_tcp(Spawn::Process, TransportKind::Tcp, ProbeStretch::Minimal)]
#[case::process_real_grpc(Spawn::Process, TransportKind::Grpc, ProbeStretch::Minimal)]
fn a_removal_that_would_lose_the_quorum_is_refused(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] probe_stretch: ProbeStretch,
) {
    // Arrange
    let cluster = concluded("refused", 3, 3, spawn, transport, probe_stretch);

    // Act
    cluster.request_removal(COORDINATOR, 1);

    // Assert
    assert_eq!(
        cluster.snapshot(COORDINATOR).reconfiguration,
        Reconfiguration::None
    );
    assert!((0..3).all(|index| cluster.snapshot(index).member_count == 3));
}

#[rstest]
#[case::task_real_inmemory(Spawn::Task, TransportKind::InMemory, ProbeStretch::Generous)]
#[case::task_real_channels(Spawn::Task, TransportKind::Channels, ProbeStretch::Generous)]
#[case::task_real_tcp(Spawn::Task, TransportKind::Tcp, ProbeStretch::Generous)]
#[case::task_real_grpc(Spawn::Task, TransportKind::Grpc, ProbeStretch::Generous)]
#[case::thread_real_inmemory(Spawn::Thread, TransportKind::InMemory, ProbeStretch::Minimal)]
#[case::thread_real_channels(Spawn::Thread, TransportKind::Channels, ProbeStretch::Minimal)]
#[case::thread_real_tcp(Spawn::Thread, TransportKind::Tcp, ProbeStretch::Minimal)]
#[case::thread_real_grpc(Spawn::Thread, TransportKind::Grpc, ProbeStretch::Minimal)]
#[case::process_real_tcp(Spawn::Process, TransportKind::Tcp, ProbeStretch::Minimal)]
#[case::process_real_grpc(Spawn::Process, TransportKind::Grpc, ProbeStretch::Minimal)]
fn a_removed_node_leaves_the_cluster(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] probe_stretch: ProbeStretch,
) {
    // Arrange
    let mut cluster = concluded("left", 4, 2, spawn, transport, probe_stretch);
    cluster.request_removal(COORDINATOR, 3);
    cluster.poll_until_removal_proposed(COORDINATOR, 3);
    cluster.commit_removal(3, None);

    // Act
    cluster.silence(3);
    cluster.settle(SETTLE_ROUNDS_PER_STRETCH * probe_stretch.factor() as usize);

    // Assert
    assert_eq!(cluster.node_state(3), PeerState::Removed);
    for index in 0..3 {
        let snapshot = cluster.snapshot(index);
        assert_eq!(snapshot.member_count, 3);
        assert!(
            snapshot.suspected.is_empty(),
            "{index}: {:?}",
            snapshot.suspected
        );
    }
}

#[rstest]
#[case::task_real_inmemory(Spawn::Task, TransportKind::InMemory, ProbeStretch::Generous)]
#[case::task_real_channels(Spawn::Task, TransportKind::Channels, ProbeStretch::Generous)]
#[case::task_real_tcp(Spawn::Task, TransportKind::Tcp, ProbeStretch::Generous)]
#[case::task_real_grpc(Spawn::Task, TransportKind::Grpc, ProbeStretch::Generous)]
#[case::thread_real_inmemory(Spawn::Thread, TransportKind::InMemory, ProbeStretch::Minimal)]
#[case::thread_real_channels(Spawn::Thread, TransportKind::Channels, ProbeStretch::Minimal)]
#[case::thread_real_tcp(Spawn::Thread, TransportKind::Tcp, ProbeStretch::Minimal)]
#[case::thread_real_grpc(Spawn::Thread, TransportKind::Grpc, ProbeStretch::Minimal)]
#[case::process_real_tcp(Spawn::Process, TransportKind::Tcp, ProbeStretch::Minimal)]
#[case::process_real_grpc(Spawn::Process, TransportKind::Grpc, ProbeStretch::Minimal)]
fn a_silenced_member_is_suspected_then_removed(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] probe_stretch: ProbeStretch,
) {
    // Arrange
    let mut cluster = concluded("silenced", 4, 3, spawn, transport, probe_stretch);
    cluster.silence(3);
    cluster.poll_until(|cluster| cluster.snapshot(COORDINATOR).suspected.contains(&3));

    // Act
    cluster.request_removal(COORDINATOR, 3);
    cluster.poll_until_removal_proposed(COORDINATOR, 3);
    cluster.commit_removal(3, None);

    // Assert
    for index in 0..3 {
        let snapshot = cluster.snapshot(index);
        assert_eq!(snapshot.member_count, 3);
        assert_eq!(snapshot.quorum, Quorum::Held);
    }
}

#[rstest]
#[case::task_real_inmemory(Spawn::Task, TransportKind::InMemory, ProbeStretch::Generous)]
#[case::task_real_channels(Spawn::Task, TransportKind::Channels, ProbeStretch::Generous)]
#[case::task_real_tcp(Spawn::Task, TransportKind::Tcp, ProbeStretch::Generous)]
#[case::task_real_grpc(Spawn::Task, TransportKind::Grpc, ProbeStretch::Generous)]
#[case::thread_real_inmemory(Spawn::Thread, TransportKind::InMemory, ProbeStretch::Minimal)]
#[case::thread_real_channels(Spawn::Thread, TransportKind::Channels, ProbeStretch::Minimal)]
#[case::thread_real_tcp(Spawn::Thread, TransportKind::Tcp, ProbeStretch::Minimal)]
#[case::thread_real_grpc(Spawn::Thread, TransportKind::Grpc, ProbeStretch::Minimal)]
#[case::process_real_tcp(Spawn::Process, TransportKind::Tcp, ProbeStretch::Minimal)]
#[case::process_real_grpc(Spawn::Process, TransportKind::Grpc, ProbeStretch::Minimal)]
fn an_aborted_removal_keeps_the_member(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] probe_stretch: ProbeStretch,
) {
    // Arrange
    let mut cluster = concluded("aborted", 4, 3, spawn, transport, probe_stretch);
    cluster.request_removal(COORDINATOR, 3);
    cluster.poll_until_removal_proposed(COORDINATOR, 3);

    // Act
    cluster.abort_removal(COORDINATOR, 3);

    // Assert
    assert_eq!(
        cluster.snapshot(COORDINATOR).reconfiguration,
        Reconfiguration::None
    );
    assert!((0..4).all(|index| cluster.snapshot(index).member_count == 4));
}

fn concluded(
    scenario: &str,
    size: usize,
    required: usize,
    spawn: Spawn,
    transport: TransportKind,
    probe_stretch: ProbeStretch,
) -> Cluster {
    let mut cluster = ClusterBuilder::new(size, required, real_time())
        .spawn(spawn)
        .transport(transport)
        .suspicion_threshold(SUSPICION_THRESHOLD)
        .probe_stretch(probe_stretch)
        .log_dir(scenario_log(scenario, spawn, transport))
        .build();
    cluster.poll_until_bootstrapped();
    cluster
}

fn scenario_log(scenario: &str, spawn: Spawn, transport: TransportKind) -> PathBuf {
    log_dir(&format!("removal-{scenario}-{spawn:?}-{transport:?}"))
}
