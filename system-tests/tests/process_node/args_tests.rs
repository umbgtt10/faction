// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::process_node::args::Args;
use faction_system_tests::transport_kind::TransportKind;
use std::net::SocketAddr;
use std::path::PathBuf;

fn addr(text: &str) -> SocketAddr {
    text.parse().unwrap()
}

fn args(extra: &[&str]) -> Args {
    let required = [
        "faction-node",
        "--peer-id",
        "1",
        "--peers",
        "0,1",
        "--required",
        "2",
        "--transport",
        "tcp",
        "--timer-delay",
        "20",
        "--listen-addr",
        "127.0.0.1:4001",
        "--peer-addrs",
        "0=127.0.0.1:4000,1=127.0.0.1:4001",
    ];
    Args::new(
        required
            .iter()
            .chain(extra)
            .map(|arg| String::from(*arg))
            .collect(),
    )
}

#[test]
fn parse_reads_every_required_argument() {
    // Arrange
    let args = args(&[]);

    // Act
    let config = args.parse();

    // Assert
    assert_eq!(config.peer_id, 1);
    assert_eq!(config.peers, vec![0, 1]);
    assert_eq!(config.required, 2);
    assert_eq!(config.transport, TransportKind::Tcp);
    assert_eq!(config.timer_delay_ms, 20);
    assert_eq!(config.listen_addr, addr("127.0.0.1:4001"));
    assert_eq!(
        config.peer_addrs,
        vec![(0, addr("127.0.0.1:4000")), (1, addr("127.0.0.1:4001"))]
    );
    assert_eq!(config.freshness_margin, 2);
    assert_eq!(config.log_path, None);
}

#[test]
fn parse_reads_the_optional_arguments_and_a_grpc_transport() {
    // Arrange
    let args = args(&[
        "--freshness-margin",
        "7",
        "--log-path",
        "logs/node-1.jsonl",
        "--transport",
        "grpc",
    ]);

    // Act
    let config = args.parse();

    // Assert
    assert_eq!(config.freshness_margin, 7);
    assert_eq!(config.log_path, Some(PathBuf::from("logs/node-1.jsonl")));
    assert_eq!(config.transport, TransportKind::Grpc);
}

#[test]
#[should_panic(expected = "unknown argument: --bogus")]
fn parse_unknown_argument_panics() {
    // Arrange
    let args = args(&["--bogus"]);

    // Act & Assert
    let _ = args.parse();
}

#[test]
#[should_panic(expected = "unknown transport: pigeon")]
fn parse_unknown_transport_panics() {
    // Arrange
    let args = args(&["--transport", "pigeon"]);

    // Act & Assert
    let _ = args.parse();
}

#[test]
#[should_panic(expected = "missing --peer-id")]
fn parse_without_a_peer_id_panics() {
    // Arrange
    let args = Args::new(vec![String::from("faction-node")]);

    // Act & Assert
    let _ = args.parse();
}
