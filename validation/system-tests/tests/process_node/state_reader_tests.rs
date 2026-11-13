// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;
use faction_system_tests::node_snapshot::NodeSnapshot;
use faction_system_tests::process_node::state_reader::StateReader;
use rstest::rstest;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;

fn read(output: &'static str) -> (NodeSnapshot, Receiver<()>) {
    let snapshot = Arc::new(Mutex::new(NodeSnapshot {
        peer_state: PeerState::Pinging,
        member_count: 9,
    }));
    let (acks, ack_receiver) = channel();
    StateReader::new(snapshot.clone(), acks)
        .spawn(Cursor::new(output))
        .join()
        .unwrap();
    let read = *snapshot.lock().unwrap();
    (read, ack_receiver)
}

#[rstest]
#[case("state Fresh 2", PeerState::Fresh)]
#[case("state Pinging 2", PeerState::Pinging)]
#[case("state Collecting 2", PeerState::Collecting)]
#[case("state Bootstrapped 2", PeerState::Bootstrapped)]
#[case("state TimedOut 2", PeerState::TimedOut)]
#[case("state Weird 2", PeerState::Fresh)]
fn spawn_reads_a_state_line_into_the_snapshot(
    #[case] line: &'static str,
    #[case] peer_state: PeerState,
) {
    // Arrange & Act
    let (snapshot, _) = read(line);

    // Assert
    assert_eq!(
        snapshot,
        NodeSnapshot {
            peer_state,
            member_count: 2,
        }
    );
}

#[test]
fn spawn_forwards_an_ack() {
    // Arrange & Act
    let (_, acks) = read("ack\n");

    // Assert
    assert_eq!(acks.try_recv(), Ok(()));
}

#[test]
fn spawn_ignores_any_other_line() {
    // Arrange & Act
    let (snapshot, acks) = read("hello\nstate\nstate Pinging many\n");

    // Assert
    assert_eq!(snapshot.member_count, 9);
    assert!(acks.try_recv().is_err());
}
