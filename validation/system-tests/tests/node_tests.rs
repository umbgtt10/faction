// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::fake_time;
use crate::support::in_memory_node;
use crate::support::real_time;
use crate::support::recipe;
use crate::traits::monotonic_clock_tests::FakeMonotonicClock;
use faction::peer_state::PeerState;
use faction_system_tests::infra::network::tcp_readiness::TcpReadiness;
use faction_system_tests::infra::transport::in_memory::in_memory_transport::InMemoryTransport;
use faction_system_tests::node::Node;
use faction_system_tests::process_spec::ProcessSpec;
use faction_system_tests::transport_kind::TransportKind;
use std::cell::RefCell;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::process::Command;
use std::process::Stdio;
use std::rc::Rc;
use std::sync::Arc;
use std::thread::yield_now;
use std::time::Duration;

fn bootstrapped(node: &Node) {
    while node.peer_state() != PeerState::Bootstrapped {
        yield_now();
    }
}

fn free_addr() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap()
}

fn process_node() -> Node {
    let addr = free_addr();
    spec().launch(0, &[0], &[(0, addr)], addr)
}

fn spec() -> ProcessSpec {
    ProcessSpec::new(
        String::from(env!("CARGO_BIN_EXE_faction-node")),
        TransportKind::Tcp,
        1,
        1,
        Duration::from_millis(10),
        100,
        1,
        None,
        TcpReadiness::new(real_time(), Duration::from_secs(30)),
    )
}

fn task_node() -> (Node, Arc<FakeMonotonicClock>) {
    let (time, clock) = fake_time();
    let node = in_memory_node(&[0, 1, 2], 2, &time);
    (Node::task(Rc::new(RefCell::new(node))), clock)
}

fn thread_node() -> Node {
    let time = real_time();
    let recipe = recipe(1, &time);
    let transport = InMemoryTransport::new_mesh(&[0]).remove(0);
    Node::spawn_thread(
        move || recipe.build(0, vec![0], Box::new(transport)),
        time,
        Duration::from_millis(1),
    )
}

#[test]
fn add_peer_address_of_a_process_node_is_acknowledged() {
    // Arrange
    let mut node = process_node();
    bootstrapped(&node);

    // Act
    node.add_peer_address(1, free_addr());

    // Assert
    assert_eq!(node.member_count(), 1);
    node.shutdown();
}

#[test]
fn admit_of_a_process_node_adds_a_member() {
    // Arrange
    let mut node = process_node();
    bootstrapped(&node);

    // Act
    node.admit(4);

    // Assert
    assert_eq!(node.member_count(), 2);
    node.shutdown();
}

#[test]
fn admit_of_a_thread_node_adds_a_member() {
    // Arrange
    let mut node = thread_node();

    // Act
    node.admit(4);

    // Assert
    assert_eq!(node.member_count(), 2);
    node.shutdown();
}

#[test]
fn deny_of_a_process_node_leaves_the_members() {
    // Arrange
    let mut node = process_node();
    bootstrapped(&node);

    // Act
    node.deny(4);

    // Assert
    assert_eq!(node.member_count(), 1);
    node.shutdown();
}

#[test]
fn deny_of_a_task_node_leaves_the_members() {
    // Arrange
    let (node, _) = task_node();

    // Act
    node.deny(4);

    // Assert
    assert_eq!(node.member_count(), 3);
}

#[test]
fn deny_of_a_thread_node_leaves_the_members() {
    // Arrange
    let mut node = thread_node();

    // Act
    node.deny(4);

    // Assert
    assert_eq!(node.member_count(), 1);
    node.shutdown();
}

#[test]
fn expire_deadline_of_a_task_node_while_pinging_reports_timed_out() {
    // Arrange
    let (node, clock) = task_node();
    node.start();
    clock.advance(Duration::from_millis(10));
    node.step();

    // Act
    node.expire_deadline();

    // Assert
    assert_eq!(node.peer_state(), PeerState::TimedOut);
}

#[test]
fn launch_newcomer_starts_a_process_waiting_with_no_members() {
    // Arrange
    let addr = free_addr();

    // Act
    let mut node = spec().launch_newcomer(5, &[(5, addr)], addr);
    node.add_peer_address(0, free_addr());

    // Assert
    assert_eq!(node.snapshot().member_count, 0);
    assert_eq!(node.peer_state(), PeerState::Fresh);
    node.shutdown();
}

#[test]
fn member_count_of_a_task_node_counts_its_peers() {
    // Arrange
    let (node, _) = task_node();

    // Act
    let members = node.member_count();

    // Assert
    assert_eq!(members, 3);
}

#[test]
fn peer_state_of_a_single_process_node_reaches_bootstrapped() {
    // Arrange
    let mut node = process_node();

    // Act
    bootstrapped(&node);

    // Assert
    assert_eq!(node.peer_state(), PeerState::Bootstrapped);
    node.shutdown();
}

#[test]
fn peer_state_of_a_single_thread_node_reaches_bootstrapped() {
    // Arrange
    let mut node = thread_node();

    // Act
    bootstrapped(&node);

    // Assert
    assert_eq!(node.peer_state(), PeerState::Bootstrapped);
    node.shutdown();
}

#[test]
fn process_of_a_child_that_reports_nothing_stays_fresh() {
    // Arrange
    let child = Command::new(env!("CARGO_BIN_EXE_faction-node"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    // Act
    let mut node = Node::process(child);

    // Assert
    assert_eq!(node.peer_state(), PeerState::Fresh);
    assert_eq!(node.member_count(), 0);
    node.shutdown();
}

#[test]
fn process_wraps_a_launched_child() {
    // Arrange & Act
    let mut node = process_node();

    // Assert
    assert!(matches!(node, Node::Process { .. }));
    node.shutdown();
}

#[test]
fn request_join_of_a_process_node_leaves_the_members() {
    // Arrange
    let mut node = process_node();
    bootstrapped(&node);

    // Act
    node.request_join(4);

    // Assert
    assert_eq!(node.member_count(), 1);
    node.shutdown();
}

#[test]
fn request_join_of_a_task_node_leaves_the_members() {
    // Arrange
    let (node, _) = task_node();

    // Act
    node.request_join(4);

    // Assert
    assert_eq!(node.member_count(), 3);
}

#[test]
fn request_join_of_a_thread_node_leaves_the_members() {
    // Arrange
    let mut node = thread_node();

    // Act
    node.request_join(4);

    // Assert
    assert_eq!(node.member_count(), 1);
    node.shutdown();
}

#[test]
fn shutdown_of_a_thread_node_stops_its_thread() {
    // Arrange
    let mut node = thread_node();

    // Act
    node.shutdown();

    // Assert
    assert!(matches!(node, Node::Thread { handle: None, .. }));
}

#[test]
fn spawn_thread_runs_the_node_on_a_thread_of_its_own() {
    // Arrange & Act
    let mut node = thread_node();

    // Assert
    assert!(matches!(
        node,
        Node::Thread {
            handle: Some(_),
            ..
        }
    ));
    node.shutdown();
}

#[test]
fn start_and_step_of_a_task_node_reach_pinging() {
    // Arrange
    let (node, clock) = task_node();

    // Act
    node.start();
    clock.advance(Duration::from_millis(10));
    node.step();

    // Assert
    assert_eq!(node.peer_state(), PeerState::Pinging);
}

#[test]
fn task_wraps_a_node_run_in_place() {
    // Arrange & Act
    let (node, _) = task_node();

    // Assert
    assert_eq!(node.peer_state(), PeerState::Fresh);
}
