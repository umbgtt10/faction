// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

use crate::engine::soak_limit::SoakLimit;
use crate::usage::Usage;

const DEFAULT_SOAK: Duration = Duration::from_secs(3_600);
const SECONDS_AN_HOUR: u64 = 3_600;
const SECONDS_A_MINUTE: u64 = 60;

// What the chaos binary is asked: to replay one seed, its nodes' logs
// written; or to soak, for as many scenarios as `CHAOS_SCENARIOS` says, else
// for as long as `CHAOS_DURATION` says -- seconds, or with `s`, `m` or `h` --
// else for an hour, from the seed `CHAOS_SEED` says, else a fresh one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invocation {
    Replay { seed: u64 },
    Soak { from: Option<u64>, limit: SoakLimit },
}

impl Invocation {
    // The invocation `args` -- the words after the binary's name -- ask for,
    // a soak's settings read through `variable`.
    pub fn parse(
        args: &[String],
        variable: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Self, Usage> {
        match args {
            [command, seed] if command == "replay" => seed
                .parse()
                .map(|seed| Self::Replay { seed })
                .map_err(|_| Usage),
            [command] if command == "soak" => Ok(Self::Soak {
                from: Self::seed(variable)?,
                limit: Self::limit(variable)?,
            }),
            _ => Err(Usage),
        }
    }

    fn duration(text: &str) -> Result<Duration, Usage> {
        let (number, unit) = match text.char_indices().last() {
            Some((at, 'h')) => (&text[..at], SECONDS_AN_HOUR),
            Some((at, 'm')) => (&text[..at], SECONDS_A_MINUTE),
            Some((at, 's')) => (&text[..at], 1),
            _ => (text, 1),
        };
        number
            .parse::<u64>()
            .map(|count| Duration::from_secs(count * unit))
            .map_err(|_| Usage)
    }

    fn limit(variable: &dyn Fn(&str) -> Option<String>) -> Result<SoakLimit, Usage> {
        if let Some(count) = variable("CHAOS_SCENARIOS") {
            return count.parse().map(SoakLimit::Scenarios).map_err(|_| Usage);
        }
        variable("CHAOS_DURATION").map_or(Ok(SoakLimit::For(DEFAULT_SOAK)), |text| {
            Self::duration(&text).map(SoakLimit::For)
        })
    }

    fn seed(variable: &dyn Fn(&str) -> Option<String>) -> Result<Option<u64>, Usage> {
        variable("CHAOS_SEED")
            .map(|seed| seed.parse().map_err(|_| Usage))
            .transpose()
    }
}
