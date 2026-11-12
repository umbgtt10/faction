// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_dir;
use std::fs::remove_dir_all;
use std::fs::remove_file;
use std::fs::write;
use std::path::Path;
use std::path::PathBuf;
use std::process::id;
use xtask::artifacts::run_log_directory::RunLogDirectory;
use xtask::artifacts::run_log_failure::RunLogFailure;

const RUN: &str = "2026-08-25_16-12-02";

fn make_runs(root: &Path, names: &[&str]) {
    for name in names {
        create_dir_all(root.join(name)).expect("failed to create a run directory");
    }
}

fn names_in(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = read_dir(root)
        .expect("failed to read scratch root")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

// Under the platform's own temp directory rather than a path spelled out here,
// so the suite runs the same on all three runners. Named after the test and the
// process, not the clock: these tests share one process, where their names
// differ, and a run beside them has its own id. Cleared first, so a directory a
// failed run left under the same name is not mistaken for this test's, and not
// created, because prepare makes the log root itself.
fn scratch_root(name: &str) -> PathBuf {
    let root = temp_dir().join(format!("run-log-directory-tests-{name}-{}", id()));
    remove_dir_all(&root).ok();
    root
}

#[test]
fn prepare_creates_the_run_directory_and_returns_its_path() {
    // Arrange
    let root = scratch_root("creates");
    create_dir_all(&root).expect("failed to create scratch root");

    // Act
    let created = RunLogDirectory::new(root.clone(), 30)
        .prepare(RUN)
        .expect("prepare succeeds");

    // Assert
    assert_eq!(created, root.join(RUN));
    assert!(created.is_dir());
    remove_dir_all(&root).ok();
}

// slotgate writes one log per job. Were this run to write where the last one
// did, a log left by a job that no longer exists would read afterwards as a
// result of this run. Each run's directory is new, so it holds nothing until
// this run writes it -- and the last run keeps what it wrote.
#[test]
fn prepare_hands_out_an_empty_run_directory_while_earlier_runs_keep_their_logs() {
    // Arrange
    let root = scratch_root("fresh");
    make_runs(&root, &["2026-08-20_10-00-00"]);
    let stale = root
        .join("2026-08-20_10-00-00")
        .join("cluster_three_node_partition.log");
    write(&stale, "a job that no longer exists").expect("failed to write a stale log");

    // Act
    let created = RunLogDirectory::new(root.clone(), 30)
        .prepare(RUN)
        .expect("prepare succeeds");

    // Assert
    assert_eq!(names_in(&created), Vec::<String>::new());
    assert!(stale.exists());
    remove_dir_all(&root).ok();
}

#[test]
fn prepare_keeps_the_newest_runs_and_removes_the_rest() {
    // Arrange
    let root = scratch_root("rotates");
    make_runs(
        &root,
        &[
            "2026-08-20_10-00-00",
            "2026-08-21_10-00-00",
            "2026-08-22_10-00-00",
        ],
    );

    // Act -- the new run counts towards the two, so two of the three go
    RunLogDirectory::new(root.clone(), 2)
        .prepare(RUN)
        .expect("prepare succeeds");

    // Assert
    assert_eq!(names_in(&root), vec!["2026-08-22_10-00-00", RUN]);
    remove_dir_all(&root).ok();
}

// Anything under logs/ that is not stamped was put there by hand or by another
// tool -- the harness's own failure logs carry a millisecond stamp, and stage 5
// wrote to slotgate-stage5 before it kept its runs. The rotation is not
// entitled to delete any of them, however old they are.
#[test]
fn prepare_leaves_directories_that_are_not_run_stamps_alone() {
    // Arrange
    let root = scratch_root("unstamped");
    make_runs(
        &root,
        &[
            "2026-08-20_10-00-00",
            "2026-08-21_10-00-00",
            "2026-08-19_10-00-00-123",
            "slotgate-stage5",
            "keep-me",
            "2026-08-22",
        ],
    );

    // Act
    RunLogDirectory::new(root.clone(), 1)
        .prepare(RUN)
        .expect("prepare succeeds");

    // Assert -- both stamped runs went, and none of the four others did
    assert_eq!(
        names_in(&root),
        vec![
            "2026-08-19_10-00-00-123",
            "2026-08-22",
            RUN,
            "keep-me",
            "slotgate-stage5"
        ]
    );
    remove_dir_all(&root).ok();
}

#[test]
fn prepare_leaves_loose_files_in_the_log_root_alone() {
    // Arrange
    let root = scratch_root("files");
    make_runs(&root, &["2026-08-20_10-00-00"]);
    write(root.join("2026-08-19_10-00-00"), "a file, not a run")
        .expect("failed to write a loose file");

    // Act
    RunLogDirectory::new(root.clone(), 1)
        .prepare(RUN)
        .expect("prepare succeeds");

    // Assert -- the stamped directory went; the identically named file did not
    assert_eq!(names_in(&root), vec!["2026-08-19_10-00-00", RUN]);
    remove_dir_all(&root).ok();
}

// Two runs stamped in the same second would share a directory, and the second
// would read the first's logs as its own. prepare refuses one it did not make
// rather than writing into it.
#[test]
fn prepare_of_a_name_already_taken_fails_rather_than_reusing_its_logs() {
    // Arrange
    let root = scratch_root("taken");
    make_runs(&root, &[RUN]);
    let stale = root.join(RUN).join("cluster_election_tests.log");
    write(&stale, "the run that took this name first").expect("failed to write a stale log");

    // Act
    let result = RunLogDirectory::new(root.clone(), 30).prepare(RUN);

    // Assert
    assert!(
        matches!(&result, Err(RunLogFailure::Create { path, .. }) if path == &root.join(RUN)),
        "{result:?}"
    );
    assert!(stale.exists());
    remove_dir_all(&root).ok();
}

// A fresh checkout has no logs/ at all, and stage 5 is the first thing to want
// it.
#[test]
fn prepare_on_a_missing_log_root_creates_it_with_the_run() {
    // Arrange
    let root = scratch_root("missing");

    // Act
    let created = RunLogDirectory::new(root.clone(), 30)
        .prepare(RUN)
        .expect("prepare succeeds");

    // Assert
    assert_eq!(created, root.join(RUN));
    assert_eq!(names_in(&root), vec![RUN]);
    remove_dir_all(&root).ok();
}

#[test]
fn prepare_on_an_empty_log_root_returns_only_the_new_run() {
    // Arrange
    let root = scratch_root("empty");
    create_dir_all(&root).expect("failed to create scratch root");

    // Act
    RunLogDirectory::new(root.clone(), 30)
        .prepare(RUN)
        .expect("prepare succeeds");

    // Assert
    assert_eq!(names_in(&root), vec![RUN]);
    remove_dir_all(&root).ok();
}

#[test]
fn prepare_removes_a_run_directory_together_with_its_contents() {
    // Arrange
    let root = scratch_root("recursive");
    make_runs(&root, &["2026-08-20_10-00-00"]);
    write(
        root.join("2026-08-20_10-00-00").join("cluster.log"),
        "a job from a previous run",
    )
    .expect("failed to write a log");

    // Act
    RunLogDirectory::new(root.clone(), 1)
        .prepare(RUN)
        .expect("prepare succeeds");

    // Assert
    assert!(!root.join("2026-08-20_10-00-00").exists());
    remove_dir_all(&root).ok();
}

// A file where the log root should be: there, but nothing a run directory can
// be made in.
#[test]
fn prepare_where_a_file_stands_in_place_of_the_log_root_fails_naming_the_path() {
    // Arrange
    let root = scratch_root("file");
    write(&root, "not a directory").expect("failed to write scratch file");

    // Act
    let result = RunLogDirectory::new(root.clone(), 30).prepare(RUN);

    // Assert
    assert!(
        matches!(&result, Err(RunLogFailure::Create { path, .. }) if path == &root),
        "{result:?}"
    );
    remove_file(&root).ok();
}

#[test]
fn prepare_with_fewer_runs_than_the_limit_removes_nothing() {
    // Arrange
    let root = scratch_root("under-limit");
    make_runs(&root, &["2026-08-20_10-00-00", "2026-08-21_10-00-00"]);

    // Act
    RunLogDirectory::new(root.clone(), 30)
        .prepare(RUN)
        .expect("prepare succeeds");

    // Assert
    assert_eq!(
        names_in(&root),
        vec!["2026-08-20_10-00-00", "2026-08-21_10-00-00", RUN]
    );
    remove_dir_all(&root).ok();
}
