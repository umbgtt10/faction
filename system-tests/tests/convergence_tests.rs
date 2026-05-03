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
#[case::task_real_inmemory(Spawn::Task, TransportKind::InMemory, TimerDelay::Minimal)]
#[case::task_real_channels(Spawn::Task, TransportKind::Channels, TimerDelay::Minimal)]
#[case::task_real_tcp(Spawn::Task, TransportKind::Tcp, TimerDelay::Minimal)]
#[case::task_real_grpc(Spawn::Task, TransportKind::Grpc, TimerDelay::Minimal)]
#[case::thread_real_inmemory(Spawn::Thread, TransportKind::InMemory, TimerDelay::Moderate)]
#[case::thread_real_channels(Spawn::Thread, TransportKind::Channels, TimerDelay::Moderate)]
#[case::thread_real_tcp(Spawn::Thread, TransportKind::Tcp, TimerDelay::Moderate)]
#[case::thread_real_grpc(Spawn::Thread, TransportKind::Grpc, TimerDelay::Moderate)]
#[case::process_real_tcp(Spawn::Process, TransportKind::Tcp, TimerDelay::Generous)]
#[case::process_real_grpc(Spawn::Process, TransportKind::Grpc, TimerDelay::Generous)]
fn cluster_reaches_bootstrapped(
    #[case] spawn: Spawn,
    #[case] transport: TransportKind,
    #[case] delay: TimerDelay,
) {
    // Arrange
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let name = format!("{timestamp}_{spawn:?}_{transport:?}.jsonl").to_lowercase();

    let mut cluster = ClusterBuilder::new(5, 4)
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
