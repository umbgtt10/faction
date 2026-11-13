// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::SharedOutput;
use crate::support::fake_time;
use crate::support::in_memory_node;
use faction_system_tests::infra::network::address_book::AddressBook;
use faction_system_tests::process_node::process_command::ProcessCommand;
use faction_system_tests::process_node::process_runner::ProcessRunner;
use faction_system_tests::time_ports::TimePorts;
use faction_system_tests::traits::monotonic_clock::MonotonicClock;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::time::Duration;

fn runner(
    time: TimePorts,
    commands: Receiver<ProcessCommand>,
    idle_delay: Duration,
) -> (ProcessRunner, SharedOutput, AddressBook) {
    let output = SharedOutput::default();
    let address_book: AddressBook = Arc::new(Mutex::new(Vec::new()));
    let runner = ProcessRunner::new(
        in_memory_node(&[0, 1, 2], 2, &time),
        commands,
        address_book.clone(),
        Box::new(output.clone()),
        time,
        idle_delay,
    );
    (runner, output, address_book)
}

fn sent(commands: &[ProcessCommand]) -> (Sender<ProcessCommand>, Receiver<ProcessCommand>) {
    let (sender, receiver) = channel();
    for command in commands {
        sender.send(*command).unwrap();
    }
    (sender, receiver)
}

#[test]
fn new_writes_nothing_until_run() {
    // Arrange
    let (time, _) = fake_time();
    let (_sender, receiver) = sent(&[]);

    // Act
    let (_runner, output, _) = runner(time, receiver, Duration::from_millis(10));

    // Assert
    assert_eq!(output.text(), "");
}

#[test]
fn run_applies_a_command_then_acknowledges_it() {
    // Arrange
    let (time, _) = fake_time();
    let (_sender, receiver) = sent(&[ProcessCommand::Admit(5), ProcessCommand::Shutdown]);
    let (runner, output, _) = runner(time, receiver, Duration::from_millis(10));

    // Act
    runner.run();

    // Assert
    assert_eq!(
        output.text(),
        "state Fresh 3 2 none - off\nstate Fresh 4 2 none - off\nack\nstate Fresh 4 2 none - off\n"
    );
}

#[test]
fn run_applies_a_cut_and_a_mend_and_acknowledges_each() {
    // Arrange
    let (time, _) = fake_time();
    let (_sender, receiver) = sent(&[
        ProcessCommand::Cut(1),
        ProcessCommand::Mend(1),
        ProcessCommand::Shutdown,
    ]);
    let (runner, output, _) = runner(time, receiver, Duration::from_millis(10));

    // Act
    runner.run();

    // Assert
    assert_eq!(output.text().matches("ack").count(), 2);
}

#[test]
fn run_applies_the_addition_commands_and_acknowledges_each() {
    // Arrange
    let (time, _) = fake_time();
    let (_sender, receiver) = sent(&[
        ProcessCommand::RequestAddition(7),
        ProcessCommand::CommitAddition(7),
        ProcessCommand::AbortAddition(7),
        ProcessCommand::RequiredCount(3),
        ProcessCommand::Shutdown,
    ]);
    let (runner, output, _) = runner(time, receiver, Duration::from_millis(10));

    // Act
    runner.run();

    // Assert
    let text = output.text();
    assert_eq!(text.matches("ack").count(), 4);
    assert!(text.ends_with("state Fresh 4 3 none - off\n"), "{text}");
}

#[test]
fn run_applies_the_join_commands_without_admitting() {
    // Arrange
    let (time, _) = fake_time();
    let (_sender, receiver) = sent(&[
        ProcessCommand::RequestJoin(7),
        ProcessCommand::Deny(7),
        ProcessCommand::ExpireDeadline,
        ProcessCommand::Shutdown,
    ]);
    let (runner, output, _) = runner(time, receiver, Duration::from_millis(10));

    // Act
    runner.run();

    // Assert
    let text = output.text();
    assert_eq!(text.matches("ack").count(), 3);
    assert!(text.ends_with("state Fresh 3 2 none - off\n"), "{text}");
}

#[test]
fn run_records_a_peer_address() {
    // Arrange
    let (time, _) = fake_time();
    let addr: SocketAddr = "127.0.0.1:4005".parse().unwrap();
    let (_sender, receiver) = sent(&[ProcessCommand::Peer(5, addr), ProcessCommand::Shutdown]);
    let (runner, _, address_book) = runner(time, receiver, Duration::from_millis(10));

    // Act
    runner.run();

    // Assert
    assert_eq!(*address_book.lock().unwrap(), vec![(5, addr)]);
}

#[test]
fn run_without_commands_ends_once_its_lifetime_is_over() {
    // Arrange
    let (time, clock) = fake_time();
    let (_sender, receiver) = sent(&[]);
    let (runner, output, _) = runner(time, receiver, Duration::from_secs(10));

    // Act
    runner.run();

    // Assert
    assert!(clock.now() >= Duration::from_secs(30), "{:?}", clock.now());
    assert!(output.text().starts_with("state Fresh 3 2 none - off\n"));
}
