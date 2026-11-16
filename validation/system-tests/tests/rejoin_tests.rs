// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Epochs and rejoin across every spawn and transport: the harness plays the
// layer that commits, numbering its commits, and node 0 coordinates. A node
// restarted from genesis, behind, is answered with the membership and catches
// up; one restarted from a snapshot starts at its epoch; a removed node a
// coordinator invites back joins as a newcomer; a partitioned node the layer
// above did not reach is heard again behind and never suspected for it; and a
// node ahead refuses readiness from behind until its peers adopt its
// membership.

use crate::support::log_dir;
use crate::support::real_time;
use faction::peer_state::PeerState;
use faction::types::Epoch;
use faction::types::PeerId;
use faction_system_tests::cluster::Cluster;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::join::committer::Committer;
use faction_system_tests::membership::Membership;
use faction_system_tests::probe_stretch::ProbeStretch;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::transport_kind::TransportKind;
use rstest::rstest;
use std::path::PathBuf;

const COORDINATOR: usize = 0;
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
fn a_node_restarted_from_a_snapshot_starts_at_its_epoch(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] probe_stretch: ProbeStretch,
) {
    // Arrange
    let mut cluster = grown("snapshot", spawn, transport, probe_stretch);

    // Act
    cluster.restart(1, membership(&[0, 1, 2, 3], 2, 1));
    cluster.poll_until(|cluster| cluster.snapshot(1).peer_state == PeerState::Bootstrapped);

    // Assert
    let snapshot = cluster.snapshot(1);
    assert_eq!((snapshot.member_count, snapshot.epoch), (4, 1));
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
fn a_node_restarted_from_genesis_catches_up_with_an_addition(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] probe_stretch: ProbeStretch,
) {
    // Arrange
    let mut cluster = grown("genesis", spawn, transport, probe_stretch);

    // Act
    cluster.restart(1, membership(&[0, 1, 2], 2, 0));
    cluster.poll_until(|cluster| {
        let snapshot = cluster.snapshot(1);
        snapshot.peer_state == PeerState::Bootstrapped && snapshot.epoch == 1
    });

    // Assert
    let snapshot = cluster.snapshot(1);
    assert_eq!((snapshot.member_count, snapshot.epoch), (4, 1));
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
fn a_partitioned_node_behind_is_heard_again_and_caught_up(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] probe_stretch: ProbeStretch,
) {
    // Arrange
    let mut cluster = concluded("partitioned", 4, 2, spawn, transport, probe_stretch);
    cluster.silence(3);
    cluster.commit_removal_missed_by(2, None, &[3]);
    cluster.poll_until(|cluster| cluster.snapshot(COORDINATOR).suspected.contains(&3));

    // Act
    cluster.unsilence(3);
    cluster.poll_until(|cluster| (0..2).all(|index| cluster.snapshot(index).suspected.is_empty()));
    let behind = cluster.snapshot(3).epoch;
    cluster.catch_up(3);
    cluster.poll_until(|cluster| cluster.snapshot(3).epoch == 1);

    // Assert
    assert_eq!(behind, 0);
    assert_eq!(cluster.snapshot(3).member_count, 3);
    assert!((0..2).all(|index| cluster.snapshot(index).suspected.is_empty()));
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
fn a_removed_node_invited_back_joins_as_a_newcomer(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] probe_stretch: ProbeStretch,
) {
    // Arrange
    let mut cluster = concluded("readded", 4, 2, spawn, transport, probe_stretch);
    cluster.request_removal(COORDINATOR, 3);
    cluster.poll_until_removal_proposed(COORDINATOR, 3);
    cluster.commit_removal(3, None);
    cluster.poll_until(|cluster| cluster.node_state(3) == PeerState::Removed);

    // Act
    cluster.request_addition(COORDINATOR, 3);
    cluster.poll_until_proposed(COORDINATOR, 3);
    cluster.commit_addition(3, None);
    cluster.poll_until_bootstrapped();

    // Assert
    for index in 0..4 {
        let snapshot = cluster.snapshot(index);
        assert_eq!(
            (snapshot.member_count, snapshot.epoch),
            (4, 2),
            "node {index}"
        );
    }
}

fn built(
    scenario: &str,
    size: usize,
    required: usize,
    spawn: Spawn,
    transport: TransportKind,
    probe_stretch: ProbeStretch,
) -> Cluster {
    ClusterBuilder::new(size, required, real_time())
        .spawn(spawn)
        .transport(transport)
        .suspicion_threshold(SUSPICION_THRESHOLD)
        .probe_stretch(probe_stretch)
        .log_dir(scenario_log(scenario, spawn, transport))
        .build()
}

fn concluded(
    scenario: &str,
    size: usize,
    required: usize,
    spawn: Spawn,
    transport: TransportKind,
    probe_stretch: ProbeStretch,
) -> Cluster {
    let mut cluster = built(scenario, size, required, spawn, transport, probe_stretch);
    cluster.poll_until_bootstrapped();
    cluster
}

// Three nodes requiring two, concluded, then node 3 added at epoch 1.
fn grown(
    scenario: &str,
    spawn: Spawn,
    transport: TransportKind,
    probe_stretch: ProbeStretch,
) -> Cluster {
    let mut cluster = concluded(scenario, 3, 2, spawn, transport, probe_stretch);
    cluster.add(
        COORDINATOR,
        3,
        Committer::Commit {
            required_count: None,
        },
    );
    cluster.poll_until_bootstrapped();
    cluster
}

fn membership(peers: &[PeerId], required_count: usize, epoch: Epoch) -> Membership {
    Membership {
        peers: peers.to_vec(),
        required_count,
        epoch,
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
fn readiness_from_behind_counts_once_its_node_adopts_the_membership(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] probe_stretch: ProbeStretch,
) {
    // Arrange
    let mut cluster = built("ahead", 3, 3, spawn, transport, probe_stretch);
    cluster.restart(0, membership(&[0, 1, 2], 3, 1));

    // Act
    cluster.poll_until(|cluster| {
        (0..3).all(|index| {
            let snapshot = cluster.snapshot(index);
            snapshot.peer_state == PeerState::Bootstrapped && snapshot.epoch == 1
        })
    });

    // Assert
    assert!((0..3).all(|index| cluster.snapshot(index).member_count == 3));
}

fn scenario_log(scenario: &str, spawn: Spawn, transport: TransportKind) -> PathBuf {
    log_dir(&format!("rejoin-{scenario}-{spawn:?}-{transport:?}"))
}
