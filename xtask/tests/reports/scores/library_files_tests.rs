// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::reports::scores::library_files::LibraryFiles;

const LIBRARY: [&str; 2] = ["core", "protocol"];

fn listing(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|path| String::from(*path)).collect()
}

#[test]
fn crates_of_a_listing_leaves_out_a_crate_with_no_source_file() {
    // Arrange
    let files = listing(&[
        "core/src/lib.rs",
        "protocol/Cargo.toml",
        "protocol/tests/all_tests.rs",
    ]);

    // Act
    let crates = LibraryFiles::crates(&LIBRARY, &files);

    // Assert
    assert_eq!(crates, ["core"]);
}

#[test]
fn crates_of_a_listing_names_each_crate_with_a_source_file_once_in_the_library_order() {
    // Arrange
    let files = listing(&[
        "protocol/src/protocol.rs",
        "core/Cargo.toml",
        "core/src/lib.rs",
        "core/src/states/pinging.rs",
        "protocol/tests/protocol_tests.rs",
    ]);

    // Act
    let crates = LibraryFiles::crates(&LIBRARY, &files);

    // Assert
    assert_eq!(crates, ["core", "protocol"]);
}

#[test]
fn crates_of_a_listing_reads_only_the_library_crates() {
    // Arrange
    let files = listing(&[
        "core-validation/src/lib.rs",
        "protocol-validation/src/lib.rs",
        "system-tests/src/lib.rs",
        "protocol/src/lib.rs",
    ]);

    // Act
    let crates = LibraryFiles::crates(&LIBRARY, &files);

    // Assert
    assert_eq!(crates, ["protocol"]);
}

#[test]
fn crates_of_a_listing_with_no_source_is_empty() {
    // Arrange
    let files = listing(&["core/Cargo.toml", "protocol/README.md"]);

    // Act
    let crates = LibraryFiles::crates(&LIBRARY, &files);

    // Assert
    assert!(crates.is_empty(), "{crates:?}");
}
