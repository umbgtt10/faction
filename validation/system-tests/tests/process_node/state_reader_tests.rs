// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::reconfiguration::Reconfiguration;
use faction_system_tests::infra::proposal::Proposal;
use faction_system_tests::node_snapshot::NodeSnapshot;
use faction_system_tests::process_node::state_reader::StateReader;
use rstest::rstest;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;

fn fresh() -> Arc<Mutex<NodeSnapshot>> {
    Arc::new(Mutex::new(NodeSnapshot {
        peer_state: PeerState::Fresh,
        member_count: 0,
        members: Vec::new(),
        required_count: 0,
        reconfigurations: Vec::new(),
        suspected: Vec::new(),
        quorum: Quorum::NotTracked,
        epoch: 0,
    }))
}

fn read(output: &'static str) -> (NodeSnapshot, Receiver<()>) {
    let snapshot = Arc::new(Mutex::new(NodeSnapshot {
        peer_state: PeerState::Pinging,
        member_count: 9,
        members: vec![9],
        required_count: 9,
        reconfigurations: vec![Reconfiguration::AdditionAcknowledged { peer_id: 9 }],
        suspected: vec![9],
        quorum: Quorum::Lost,
        epoch: 0,
    }));
    let (acks, ack_receiver) = channel();
    StateReader::new(snapshot.clone(), Arc::new(Mutex::new(Vec::new())), acks)
        .spawn(Cursor::new(output))
        .join()
        .unwrap();
    let read = snapshot.lock().unwrap().clone();
    (read, ack_receiver)
}

#[rstest]
#[case("state Fresh 2 3 none - off 5 0,1", PeerState::Fresh)]
#[case("state Pinging 2 3 none - off 5 0,1", PeerState::Pinging)]
#[case("state Collecting 2 3 none - off 5 0,1", PeerState::Collecting)]
#[case("state Bootstrapped 2 3 none - off 5 0,1", PeerState::Bootstrapped)]
#[case("state TimedOut 2 3 none - off 5 0,1", PeerState::TimedOut)]
#[case("state Removed 2 3 none - off 5 0,1", PeerState::Removed)]
#[case("state Weird 2 3 none - off 5 0,1", PeerState::Fresh)]
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
            members: vec![0, 1],
            required_count: 3,
            reconfigurations: Vec::new(),
            suspected: Vec::new(),
            quorum: Quorum::NotTracked,
            epoch: 5,
        }
    );
}

#[rstest]
#[case("proposal addition 7 3\n", vec![Proposal::Addition { peer_id: 7, epoch: 3 }])]
#[case("proposal removal 2 1\nproposal removal 2 4\n", vec![Proposal::Removal { peer_id: 2, epoch: 1 }, Proposal::Removal { peer_id: 2, epoch: 4 }])]
#[case("proposal removal 2\nproposal merger 2 1\n", vec![])]
fn spawn_reads_each_proposal_line_into_the_proposals(
    #[case] output: &'static str,
    #[case] expected: Vec<Proposal>,
) {
    // Arrange
    let proposals = Arc::new(Mutex::new(Vec::new()));
    let (acks, _) = channel();

    // Act
    StateReader::new(fresh(), proposals.clone(), acks)
        .spawn(Cursor::new(output))
        .join()
        .unwrap();

    // Assert
    assert_eq!(*proposals.lock().unwrap(), expected);
}

#[rstest]
#[case("state Bootstrapped 3 2 none - held 0 0,1", vec![])]
#[case("state Bootstrapped 3 2 requested:7 - held 0 0,1", vec![Reconfiguration::AdditionRequested { peer_id: 7 }])]
#[case("state Bootstrapped 3 2 acknowledged:7 - held 0 0,1", vec![Reconfiguration::AdditionAcknowledged { peer_id: 7 }])]
#[case("state Bootstrapped 3 2 removing:7 - held 0 0,1", vec![Reconfiguration::RemovalRequested { peer_id: 7, leaves: None }])]
#[case("state Bootstrapped 3 2 requested:7,removing:2 - held 0 0,1", vec![Reconfiguration::AdditionRequested { peer_id: 7 }, Reconfiguration::RemovalRequested { peer_id: 2, leaves: None }])]
fn spawn_reads_the_addition_of_a_state_line(
    #[case] line: &'static str,
    #[case] reconfigurations: Vec<Reconfiguration>,
) {
    // Arrange & Act
    let (snapshot, _) = read(line);

    // Assert
    assert_eq!(snapshot.reconfigurations, reconfigurations);
}

#[rstest]
#[case("state Bootstrapped 3 2 none - held 4 0,2,5", vec![0, 2, 5])]
#[case("state Removed 2 2 none - off 4 -", Vec::new())]
fn spawn_reads_the_members_of_a_state_line(#[case] line: &'static str, #[case] members: Vec<u64>) {
    // Arrange & Act
    let (snapshot, _) = read(line);

    // Assert
    assert_eq!(snapshot.members, members);
}

#[rstest]
#[case("state Bootstrapped 5 3 none - held 0 0,1", Vec::new(), Quorum::Held)]
#[case("state Bootstrapped 5 3 none 4,2 lost 0 0,1", vec![4, 2], Quorum::Lost)]
#[case("state Pinging 5 3 none - off 0 0,1", Vec::new(), Quorum::NotTracked)]
fn spawn_reads_the_suspected_members_and_the_quorum_of_a_state_line(
    #[case] line: &'static str,
    #[case] suspected: Vec<u64>,
    #[case] quorum: Quorum,
) {
    // Arrange & Act
    let (snapshot, _) = read(line);

    // Assert
    assert_eq!((snapshot.suspected, snapshot.quorum), (suspected, quorum));
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

#[test]
fn spawn_skips_a_state_line_without_its_count_and_addition() {
    // Arrange & Act
    let (snapshot, _) = read("state Fresh 2");

    // Assert
    assert_eq!(snapshot.member_count, 9);
}
