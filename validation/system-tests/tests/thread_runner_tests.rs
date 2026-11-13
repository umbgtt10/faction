// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::fake_time;
use crate::support::in_memory_node;
use faction::addition::Addition;
use faction::peer_state::PeerState;
use faction_system_tests::node_command::NodeCommand;
use faction_system_tests::node_snapshot::NodeSnapshot;
use faction_system_tests::thread_runner::ThreadRunner;
use faction_system_tests::traits::monotonic_clock::MonotonicClock;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::channel;
use std::time::Duration;

fn snapshot() -> Arc<Mutex<NodeSnapshot>> {
    Arc::new(Mutex::new(NodeSnapshot {
        peer_state: PeerState::Fresh,
        member_count: 0,
        required_count: 0,
        addition: Addition::None,
    }))
}

#[test]
fn new_publishes_nothing_until_run() {
    // Arrange
    let (time, _) = fake_time();
    let (_commands, receiver) = channel();
    let published = snapshot();

    // Act
    let _runner = ThreadRunner::new(
        in_memory_node(&[0, 1, 2], 2, &time),
        receiver,
        published.clone(),
        time,
        Duration::from_millis(10),
    );

    // Assert
    assert_eq!(published.lock().unwrap().member_count, 0);
}

#[test]
fn run_applies_the_addition_commands_and_acknowledges_each() {
    // Arrange
    let (time, _) = fake_time();
    let (commands, receiver) = channel();
    let (ack, acks) = channel();
    commands
        .send(NodeCommand::RequestAddition(7, ack.clone()))
        .unwrap();
    commands
        .send(NodeCommand::CommitAddition(7, ack.clone()))
        .unwrap();
    commands
        .send(NodeCommand::AbortAddition(7, ack.clone()))
        .unwrap();
    commands
        .send(NodeCommand::ChangeRequiredCount(3, ack))
        .unwrap();
    commands.send(NodeCommand::Shutdown).unwrap();
    let published = snapshot();
    let runner = ThreadRunner::new(
        in_memory_node(&[0, 1, 2], 2, &time),
        receiver,
        published.clone(),
        time,
        Duration::from_millis(10),
    );

    // Act
    runner.run();

    // Assert
    assert_eq!(acks.try_iter().count(), 4);
    let published = *published.lock().unwrap();
    assert_eq!(published.member_count, 4);
    assert_eq!(published.required_count, 3);
}

#[test]
fn run_applies_the_commands_before_a_shutdown_and_acknowledges_them() {
    // Arrange
    let (time, _) = fake_time();
    let (commands, receiver) = channel();
    let (ack, acks) = channel();
    commands.send(NodeCommand::Admit(5, ack)).unwrap();
    commands.send(NodeCommand::Shutdown).unwrap();
    let published = snapshot();
    let runner = ThreadRunner::new(
        in_memory_node(&[0, 1, 2], 2, &time),
        receiver,
        published.clone(),
        time,
        Duration::from_millis(10),
    );

    // Act
    runner.run();

    // Assert
    assert_eq!(acks.try_recv(), Ok(()));
    assert_eq!(published.lock().unwrap().member_count, 4);
}

#[test]
fn run_without_commands_ends_once_its_lifetime_is_over() {
    // Arrange
    let (time, clock) = fake_time();
    let (_commands, receiver) = channel();
    let published = snapshot();
    let runner = ThreadRunner::new(
        in_memory_node(&[0, 1, 2], 2, &time),
        receiver,
        published.clone(),
        time,
        Duration::from_secs(10),
    );

    // Act
    runner.run();

    // Assert
    assert!(clock.now() >= Duration::from_secs(30), "{:?}", clock.now());
    assert_eq!(published.lock().unwrap().member_count, 3);
}
