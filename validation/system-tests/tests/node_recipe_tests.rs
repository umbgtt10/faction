// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::fake_time;
use crate::support::recipe;
use faction::peer_state::PeerState;
use faction_system_tests::infra::transport::in_memory::in_memory_transport::InMemoryTransport;
use faction_system_tests::node::Node;
use faction_system_tests::node_recipe::NodeRecipe;
use faction_system_tests::spawn::Spawn;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::process::id;
use std::time::Duration;

#[test]
fn build_makes_a_fresh_node_with_the_given_peers() {
    // Arrange
    let (time, _) = fake_time();
    let transport = InMemoryTransport::new_mesh(&[0, 1, 2]).remove(0);

    // Act
    let mut node = recipe(2, &time).build(0, vec![0, 1, 2], Box::new(transport));

    // Assert
    assert_eq!(node.peer_state(), PeerState::Fresh);
    assert_eq!(node.member_count(), 3);
}

#[test]
fn new_with_a_log_directory_builds_nodes_that_log() {
    // Arrange
    let (time, _) = fake_time();
    let dir = temp_dir().join(format!("node-recipe-tests-{}", id()));
    create_dir_all(&dir).unwrap();
    let recipe = NodeRecipe::new(
        1,
        Duration::from_millis(10),
        Duration::from_secs(10),
        Some(dir.clone()),
        time,
    );
    let transport = InMemoryTransport::new_mesh(&[0]).remove(0);

    // Act
    recipe.build(0, vec![0], Box::new(transport)).start();

    // Assert
    let log = read_to_string(dir.join("node-0.jsonl")).unwrap();
    assert!(log.contains("\"event\":\"start\""), "{log}");
    remove_dir_all(&dir).ok();
}

#[test]
#[should_panic(expected = "a recipe builds in-process nodes only")]
fn node_for_a_process_panics() {
    // Arrange
    let (time, _) = fake_time();
    let transport = InMemoryTransport::new_mesh(&[0]).remove(0);

    // Act & Assert
    let _ = recipe(1, &time).node(Spawn::Process, 0, vec![0], Box::new(transport));
}

#[test]
fn node_for_a_task_runs_in_place() {
    // Arrange
    let (time, _) = fake_time();
    let transport = InMemoryTransport::new_mesh(&[0]).remove(0);

    // Act
    let node = recipe(1, &time).node(Spawn::Task, 0, vec![0], Box::new(transport));

    // Assert
    assert!(matches!(node, Node::Task { .. }));
}

#[test]
fn node_for_a_thread_runs_on_a_thread() {
    // Arrange
    let (time, _) = fake_time();
    let transport = InMemoryTransport::new_mesh(&[0]).remove(0);

    // Act
    let mut node = recipe(1, &time).node(Spawn::Thread, 0, vec![0], Box::new(transport));

    // Assert
    assert!(matches!(node, Node::Thread { .. }));
    node.shutdown();
}
