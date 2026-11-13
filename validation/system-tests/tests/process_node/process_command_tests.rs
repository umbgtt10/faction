// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::process_node::process_command::ProcessCommand;
use std::io::Cursor;
use std::net::SocketAddr;

#[test]
fn parse_admit_reads_the_peer() {
    // Arrange & Act
    let command = ProcessCommand::parse("admit 3");

    // Assert
    assert_eq!(command, Some(ProcessCommand::Admit(3)));
}

#[test]
fn parse_admit_without_a_peer_returns_none() {
    // Arrange & Act
    let command = ProcessCommand::parse("admit");

    // Assert
    assert_eq!(command, None);
}

#[test]
fn parse_deny_reads_the_peer() {
    // Arrange & Act
    let command = ProcessCommand::parse("deny 4");

    // Assert
    assert_eq!(command, Some(ProcessCommand::Deny(4)));
}

#[test]
fn parse_expire_is_expire_deadline() {
    // Arrange & Act
    let command = ProcessCommand::parse("expire");

    // Assert
    assert_eq!(command, Some(ProcessCommand::ExpireDeadline));
}

#[test]
fn parse_peer_reads_the_peer_and_its_address() {
    // Arrange
    let addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();

    // Act
    let command = ProcessCommand::parse("peer 5 127.0.0.1:4000");

    // Assert
    assert_eq!(command, Some(ProcessCommand::Peer(5, addr)));
}

#[test]
fn parse_request_join_reads_the_peer() {
    // Arrange & Act
    let command = ProcessCommand::parse("request-join 6");

    // Assert
    assert_eq!(command, Some(ProcessCommand::RequestJoin(6)));
}

#[test]
fn parse_shutdown_is_shutdown() {
    // Arrange & Act
    let command = ProcessCommand::parse("shutdown");

    // Assert
    assert_eq!(command, Some(ProcessCommand::Shutdown));
}

#[test]
fn parse_unknown_word_returns_none() {
    // Arrange & Act
    let command = ProcessCommand::parse("dance 3");

    // Assert
    assert_eq!(command, None);
}

#[test]
fn read_lines_sends_each_command_and_skips_the_rest() {
    // Arrange
    let input = Cursor::new("admit 3\nnonsense\nshutdown\n");

    // Act
    let commands = ProcessCommand::read_lines(input);

    // Assert
    assert_eq!(commands.recv(), Ok(ProcessCommand::Admit(3)));
    assert_eq!(commands.recv(), Ok(ProcessCommand::Shutdown));
    assert!(commands.recv().is_err());
}
