// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::logging::shared_writer::SharedWriter;
use std::env::temp_dir;
use std::fs::read_to_string;
use std::fs::remove_file;
use std::fs::write;
use std::path::PathBuf;
use std::process::id;

fn path(name: &str) -> PathBuf {
    temp_dir().join(format!("shared-writer-{name}-{}.jsonl", id()))
}

#[test]
fn open_appends_to_what_the_file_holds() {
    // Arrange
    let path = path("append");
    write(&path, "first\n").unwrap();
    let writer = SharedWriter::open(&path);

    // Act
    writer.write_line("second");

    // Assert
    assert_eq!(read_to_string(&path).unwrap(), "first\nsecond\n");
    remove_file(&path).ok();
}

#[test]
fn write_line_from_every_clone_lands_in_one_file() {
    // Arrange
    let path = path("clones");
    remove_file(&path).ok();
    let writer = SharedWriter::open(&path);
    let clone = writer.clone();

    // Act
    writer.write_line("one");
    clone.write_line("two");

    // Assert
    assert_eq!(read_to_string(&path).unwrap(), "one\ntwo\n");
    remove_file(&path).ok();
}
