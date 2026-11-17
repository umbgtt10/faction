// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::engine::replay::Replay;
use std::env::temp_dir;
use std::fs::remove_dir_all;
use std::process::id;

#[test]
fn folder_of_a_seed_is_named_for_it_under_the_root() {
    // Arrange
    let root = temp_dir().join(format!("replay-tests-folder-{}", id()));

    // Act
    let folder = Replay::new(7, root.clone()).folder();

    // Assert
    assert_eq!(folder, root.join("7"));
}

#[test]
fn run_replays_the_seed_with_its_nodes_logs_in_its_folder() {
    // Arrange
    let root = temp_dir().join(format!("replay-tests-run-{}", id()));
    let replay = Replay::new(3, root.clone());

    // Act
    let ended = replay.run();

    // Assert
    let written = replay.folder().join("consolidated.jsonl").exists();
    remove_dir_all(&root).ok();
    assert!(ended.is_ok(), "{ended:?}");
    assert!(written);
}
