// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::invocation::Invocation;
use faction_chaos::usage::Usage;

fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| String::from(*word)).collect()
}

#[test]
fn parse_of_anything_else_is_the_usage() {
    // Arrange
    let words = [
        args(&[]),
        args(&["replay"]),
        args(&["replay", "x"]),
        args(&["soak", "1"]),
    ];

    // Act
    let parsed: Vec<Result<Invocation, Usage>> =
        words.iter().map(|words| Invocation::parse(words)).collect();

    // Assert
    assert!(parsed.iter().all(|parsed| *parsed == Err(Usage)));
}

#[test]
fn parse_of_replay_and_a_seed_is_a_replay_of_it() {
    // Arrange
    let words = args(&["replay", "81"]);

    // Act
    let invocation = Invocation::parse(&words);

    // Assert
    assert_eq!(invocation, Ok(Invocation::Replay { seed: 81 }));
}

#[test]
fn to_string_of_the_usage_names_every_command() {
    // Arrange & Act
    let text = Usage.to_string();

    // Assert
    assert_eq!(text, "usage: faction-chaos replay <seed>");
}
