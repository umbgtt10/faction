// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::engine::soak_limit::SoakLimit;
use faction_chaos::invocation::Invocation;
use faction_chaos::usage::Usage;
use std::time::Duration;

fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| String::from(*word)).collect()
}

fn soak(variables: &[(&str, &str)]) -> Result<Invocation, Usage> {
    let lookup = |name: &str| {
        variables
            .iter()
            .find(|(variable, _)| *variable == name)
            .map(|(_, value)| String::from(*value))
    };
    Invocation::parse(&args(&["soak"]), &lookup)
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
    let parsed: Vec<Result<Invocation, Usage>> = words
        .iter()
        .map(|words| Invocation::parse(words, &|_| None))
        .collect();

    // Assert
    assert!(parsed.iter().all(|parsed| *parsed == Err(Usage)));
}

#[test]
fn parse_of_real_is_the_real_engine_s_list() {
    // Arrange
    let words = args(&["real"]);

    // Act
    let invocation = Invocation::parse(&words, &|_| None);

    // Assert
    assert_eq!(invocation, Ok(Invocation::Real));
}

#[test]
fn parse_of_replay_and_a_seed_is_a_replay_of_it() {
    // Arrange
    let words = args(&["replay", "81"]);

    // Act
    let invocation = Invocation::parse(&words, &|_| None);

    // Assert
    assert_eq!(invocation, Ok(Invocation::Replay { seed: 81 }));
}

#[test]
fn parse_of_soak_given_no_settings_soaks_an_hour_from_a_fresh_seed() {
    // Arrange & Act
    let invocation = soak(&[]);

    // Assert
    assert_eq!(
        invocation,
        Ok(Invocation::Soak {
            from: None,
            limit: SoakLimit::For(Duration::from_secs(3_600)),
        })
    );
}

#[test]
fn parse_of_soak_reads_a_count_of_scenarios_before_a_duration() {
    // Arrange & Act
    let invocation = soak(&[
        ("CHAOS_SCENARIOS", "500"),
        ("CHAOS_DURATION", "2h"),
        ("CHAOS_SEED", "7"),
    ]);

    // Assert
    assert_eq!(
        invocation,
        Ok(Invocation::Soak {
            from: Some(7),
            limit: SoakLimit::Scenarios(500),
        })
    );
}

#[test]
fn parse_of_soak_reads_a_duration_in_hours_minutes_or_seconds() {
    // Arrange
    let durations = ["2h", "15m", "90s", "45"];

    // Act
    let limits: Vec<Result<Invocation, Usage>> = durations
        .iter()
        .map(|duration| soak(&[("CHAOS_DURATION", duration)]))
        .collect();

    // Assert
    let expected: Vec<Result<Invocation, Usage>> = [7_200, 900, 90, 45]
        .iter()
        .map(|seconds| {
            Ok(Invocation::Soak {
                from: None,
                limit: SoakLimit::For(Duration::from_secs(*seconds)),
            })
        })
        .collect();
    assert_eq!(limits, expected);
}

#[test]
fn parse_of_soak_with_a_setting_it_cannot_read_is_the_usage() {
    // Arrange
    let settings = [
        [("CHAOS_SCENARIOS", "many")],
        [("CHAOS_DURATION", "2d")],
        [("CHAOS_SEED", "-1")],
    ];

    // Act
    let parsed: Vec<Result<Invocation, Usage>> =
        settings.iter().map(|setting| soak(setting)).collect();

    // Assert
    assert!(
        parsed.iter().all(|parsed| *parsed == Err(Usage)),
        "{parsed:?}"
    );
}

#[test]
fn to_string_of_the_usage_names_every_command() {
    // Arrange & Act
    let text = Usage.to_string();

    // Assert
    assert_eq!(
        text,
        "usage: faction-chaos replay <seed> | faction-chaos soak \
         (CHAOS_SCENARIOS, or CHAOS_DURATION in s, m or h; CHAOS_SEED) | faction-chaos real"
    );
}
