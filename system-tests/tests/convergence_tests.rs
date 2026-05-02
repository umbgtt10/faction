// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::path::PathBuf;

use chrono::Utc;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::timer_delay::TimerDelay;
use faction_system_tests::timer_kind::TimerKind;
use faction_system_tests::transport_kind::TransportKind;
use rstest::rstest;

#[rstest]
#[case::task_inmemory_inmemory(
    Spawn::Task,
    TimerKind::InMemory,
    TransportKind::InMemory,
    TimerDelay::Minimal
)]
#[case::task_real_inmemory(
    Spawn::Task,
    TimerKind::Real,
    TransportKind::InMemory,
    TimerDelay::Minimal
)]
#[case::task_inmemory_channels(
    Spawn::Task,
    TimerKind::InMemory,
    TransportKind::Channels,
    TimerDelay::Minimal
)]
#[case::task_real_channels(
    Spawn::Task,
    TimerKind::Real,
    TransportKind::Channels,
    TimerDelay::Minimal
)]
#[case::task_real_tcp(Spawn::Task, TimerKind::Real, TransportKind::Tcp, TimerDelay::Minimal)]
#[case::task_real_grpc(Spawn::Task, TimerKind::Real, TransportKind::Grpc, TimerDelay::Minimal)]
#[case::thread_inmemory_inmemory(
    Spawn::Thread,
    TimerKind::InMemory,
    TransportKind::InMemory,
    TimerDelay::Moderate
)]
#[case::thread_inmemory_channels(
    Spawn::Thread,
    TimerKind::InMemory,
    TransportKind::Channels,
    TimerDelay::Moderate
)]
#[case::thread_inmemory_tcp(
    Spawn::Thread,
    TimerKind::InMemory,
    TransportKind::Tcp,
    TimerDelay::Moderate
)]
#[case::thread_real_inmemory(
    Spawn::Thread,
    TimerKind::Real,
    TransportKind::InMemory,
    TimerDelay::Moderate
)]
#[case::thread_real_channels(
    Spawn::Thread,
    TimerKind::Real,
    TransportKind::Channels,
    TimerDelay::Moderate
)]
#[case::thread_real_tcp(
    Spawn::Thread,
    TimerKind::Real,
    TransportKind::Tcp,
    TimerDelay::Moderate
)]
#[case::thread_real_grpc(
    Spawn::Thread,
    TimerKind::Real,
    TransportKind::Grpc,
    TimerDelay::Moderate
)]
#[case::process_real_tcp(
    Spawn::Process,
    TimerKind::Real,
    TransportKind::Tcp,
    TimerDelay::Generous
)]
#[case::process_real_grpc(
    Spawn::Process,
    TimerKind::Real,
    TransportKind::Grpc,
    TimerDelay::Generous
)]
fn cluster_reaches_bootstrapped(
    #[case] spawn: Spawn,
    #[case] timer: TimerKind,
    #[case] transport: TransportKind,
    #[case] delay: TimerDelay,
) {
    // Arrange
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let name = format!("{timestamp}_{spawn:?}_{timer:?}_{transport:?}.jsonl").to_lowercase();

    let mut cluster = ClusterBuilder::new(5, 4)
        .spawn(spawn)
        .timer_kind(timer)
        .transport(transport)
        .timer_delay(delay)
        .log_path(PathBuf::from("logs").join(name))
        .build();

    // Act
    cluster.poll_until_bootstrapped();

    // Assert
    assert!(cluster.is_bootstrapped());
}
