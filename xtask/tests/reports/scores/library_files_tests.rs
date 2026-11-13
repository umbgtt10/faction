// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::reports::scores::library_files::LibraryFiles;

const DIRECTORY: &str = "crates";
const LIBRARY: [&str; 2] = ["core", "protocol"];

fn listing(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|path| String::from(*path)).collect()
}

#[test]
fn crates_of_a_listing_leaves_out_a_crate_with_no_source_file() {
    // Arrange
    let files = listing(&[
        "crates/core/src/lib.rs",
        "crates/protocol/Cargo.toml",
        "crates/protocol/tests/all_tests.rs",
    ]);

    // Act
    let crates = LibraryFiles::crates(DIRECTORY, &LIBRARY, &files);

    // Assert
    assert_eq!(crates, ["core"]);
}

#[test]
fn crates_of_a_listing_names_each_crate_with_a_source_file_once_in_the_library_order() {
    // Arrange
    let files = listing(&[
        "crates/protocol/src/protocol.rs",
        "crates/core/Cargo.toml",
        "crates/core/src/lib.rs",
        "crates/core/src/states/pinging.rs",
        "crates/protocol/tests/protocol_tests.rs",
    ]);

    // Act
    let crates = LibraryFiles::crates(DIRECTORY, &LIBRARY, &files);

    // Assert
    assert_eq!(crates, ["core", "protocol"]);
}

#[test]
fn crates_of_a_listing_reads_only_the_library_crates() {
    // Arrange
    let files = listing(&[
        "validation/core-validation/src/lib.rs",
        "validation/protocol-validation/src/lib.rs",
        "validation/system-tests/src/lib.rs",
        "core/src/lib.rs",
        "crates/protocol/src/lib.rs",
    ]);

    // Act
    let crates = LibraryFiles::crates(DIRECTORY, &LIBRARY, &files);

    // Assert
    assert_eq!(crates, ["protocol"]);
}

#[test]
fn crates_of_a_listing_with_no_source_is_empty() {
    // Arrange
    let files = listing(&["crates/core/Cargo.toml", "crates/protocol/README.md"]);

    // Act
    let crates = LibraryFiles::crates(DIRECTORY, &LIBRARY, &files);

    // Assert
    assert!(crates.is_empty(), "{crates:?}");
}
