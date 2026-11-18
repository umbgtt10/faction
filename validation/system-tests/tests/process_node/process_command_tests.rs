// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::fault_settings::FaultSettings;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;
use faction_system_tests::membership::Membership;
use faction_system_tests::process_node::process_command::ProcessCommand;
use serde_json::to_string;
use std::io::Cursor;
use std::net::SocketAddr;

#[test]
fn parse_abort_addition_reads_the_peer() {
    // Arrange & Act
    let command = ProcessCommand::parse("abort-addition 6");

    // Assert
    assert_eq!(command, Some(ProcessCommand::AbortAddition(6)));
}

#[test]
fn parse_abort_removal_reads_the_peer() {
    // Arrange & Act
    let command = ProcessCommand::parse("abort-removal 6");

    // Assert
    assert_eq!(command, Some(ProcessCommand::AbortRemoval(6)));
}

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
fn parse_commit_addition_reads_the_peer_and_the_epoch() {
    // Arrange & Act
    let command = ProcessCommand::parse("commit-addition 6 2");

    // Assert
    assert_eq!(command, Some(ProcessCommand::CommitAddition(6, 2)));
}

#[test]
fn parse_commit_removal_reads_the_peer_and_the_epoch() {
    // Arrange & Act
    let command = ProcessCommand::parse("commit-removal 6 3");

    // Assert
    assert_eq!(command, Some(ProcessCommand::CommitRemoval(6, 3)));
}

#[test]
fn parse_cut_reads_the_peer() {
    // Arrange & Act
    let command = ProcessCommand::parse("cut 4");

    // Assert
    assert_eq!(command, Some(ProcessCommand::Cut(4)));
}

#[test]
fn parse_decline_reads_the_peer_and_the_epoch() {
    // Arrange & Act
    let command = ProcessCommand::parse("decline 2 3");

    // Assert
    assert_eq!(command, Some(ProcessCommand::Decline(2, 3)));
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
fn parse_faults_reads_the_settings_written_as_json() {
    // Arrange
    let settings = FaultSettings {
        transport: FaultPolicy {
            mute: true,
            ..FaultPolicy::default()
        },
        ..FaultSettings::default()
    };
    let line = format!("faults {}", to_string(&settings).unwrap());

    // Act
    let command = ProcessCommand::parse(&line);

    // Assert
    assert_eq!(command, Some(ProcessCommand::Faults(Box::new(settings))));
}

#[test]
fn parse_mend_reads_the_peer() {
    // Arrange & Act
    let command = ProcessCommand::parse("mend 4");

    // Assert
    assert_eq!(command, Some(ProcessCommand::Mend(4)));
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
fn parse_request_addition_reads_the_peer() {
    // Arrange & Act
    let command = ProcessCommand::parse("request-addition 6");

    // Assert
    assert_eq!(command, Some(ProcessCommand::RequestAddition(6)));
}

#[test]
fn parse_request_join_reads_the_peer() {
    // Arrange & Act
    let command = ProcessCommand::parse("request-join 6");

    // Assert
    assert_eq!(command, Some(ProcessCommand::RequestJoin(6)));
}

#[test]
fn parse_request_removal_reads_the_peer() {
    // Arrange & Act
    let command = ProcessCommand::parse("request-removal 6");

    // Assert
    assert_eq!(command, Some(ProcessCommand::RequestRemoval(6)));
}

#[test]
fn parse_required_count_reads_the_count_and_the_epoch() {
    // Arrange & Act
    let command = ProcessCommand::parse("required-count 3 4");

    // Assert
    assert_eq!(command, Some(ProcessCommand::RequiredCount(3, 4)));
}

#[test]
fn parse_restart_reads_the_epoch_the_count_and_the_peers() {
    // Arrange & Act
    let command = ProcessCommand::parse("restart 3 2 0,5");

    // Assert
    assert_eq!(
        command,
        Some(ProcessCommand::Restart(Membership {
            peers: vec![0, 5],
            required_count: 2,
            epoch: 3,
        }))
    );
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
