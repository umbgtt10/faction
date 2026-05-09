// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::path::PathBuf;

use chrono::Utc;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::timer_delay::TimerDelay;
use faction_system_tests::transport_kind::TransportKind;
use rstest::rstest;

#[rstest]
#[case::task_real_inmemory_quorum4(Spawn::Task, TransportKind::InMemory, TimerDelay::Minimal, 4)]
#[case::task_real_inmemory_quorum5(Spawn::Task, TransportKind::InMemory, TimerDelay::Minimal, 5)]
#[case::task_real_channels_quorum4(Spawn::Task, TransportKind::Channels, TimerDelay::Minimal, 4)]
#[case::task_real_channels_quorum5(Spawn::Task, TransportKind::Channels, TimerDelay::Minimal, 5)]
#[case::task_real_tcp_quorum4(Spawn::Task, TransportKind::Tcp, TimerDelay::Minimal, 4)]
#[case::task_real_tcp_quorum5(Spawn::Task, TransportKind::Tcp, TimerDelay::Minimal, 5)]
#[case::task_real_grpc_quorum4(Spawn::Task, TransportKind::Grpc, TimerDelay::Minimal, 4)]
#[case::task_real_grpc_quorum5(Spawn::Task, TransportKind::Grpc, TimerDelay::Minimal, 5)]
#[case::thread_real_inmemory_quorum4(
    Spawn::Thread,
    TransportKind::InMemory,
    TimerDelay::Moderate,
    4
)]
#[case::thread_real_inmemory_quorum5(
    Spawn::Thread,
    TransportKind::InMemory,
    TimerDelay::Moderate,
    5
)]
#[case::thread_real_channels_quorum4(
    Spawn::Thread,
    TransportKind::Channels,
    TimerDelay::Moderate,
    4
)]
#[case::thread_real_channels_quorum5(
    Spawn::Thread,
    TransportKind::Channels,
    TimerDelay::Moderate,
    5
)]
#[case::thread_real_tcp_quorum4(Spawn::Thread, TransportKind::Tcp, TimerDelay::Moderate, 4)]
#[case::thread_real_tcp_quorum5(Spawn::Thread, TransportKind::Tcp, TimerDelay::Moderate, 5)]
#[case::thread_real_grpc_quorum4(Spawn::Thread, TransportKind::Grpc, TimerDelay::Moderate, 4)]
#[case::thread_real_grpc_quorum5(Spawn::Thread, TransportKind::Grpc, TimerDelay::Moderate, 5)]
#[case::process_real_tcp_quorum4(Spawn::Process, TransportKind::Tcp, TimerDelay::Generous, 4)]
#[case::process_real_tcp_quorum5(Spawn::Process, TransportKind::Tcp, TimerDelay::Generous, 5)]
#[case::process_real_grpc_quorum4(Spawn::Process, TransportKind::Grpc, TimerDelay::Generous, 4)]
#[case::process_real_grpc_quorum5(Spawn::Process, TransportKind::Grpc, TimerDelay::Generous, 5)]
fn cluster_reaches_bootstrapped(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] delay: TimerDelay,
    #[case] required: usize,
) {
    // Arrange
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let name = format!("{timestamp}_{spawn:?}_{transport:?}_{required:?}.jsonl").to_lowercase();

    let mut cluster = ClusterBuilder::new(5, required)
        .spawn(spawn)
        .transport(transport)
        .timer_delay(delay)
        .log_path(PathBuf::from("logs").join(name))
        .build();

    // Act
    cluster.poll_until_bootstrapped();

    // Assert
    assert!(cluster.is_bootstrapped());
}
