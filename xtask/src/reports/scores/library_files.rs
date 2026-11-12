// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

const RUST: &str = ".rs";
const SOURCE: &str = "/src/";

// The library crates that a listing of files holds source for, each once and
// in the order the library names them. A manifest or a test alone is nothing
// to score.
pub struct LibraryFiles;

impl LibraryFiles {
    pub fn crates(library: &[&str], files: &[String]) -> Vec<String> {
        library
            .iter()
            .filter(|name| {
                let source = format!("{name}{SOURCE}");
                files
                    .iter()
                    .any(|file| file.starts_with(&source) && file.ends_with(RUST))
            })
            .map(|name| String::from(*name))
            .collect()
    }
}
