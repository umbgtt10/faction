// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::usage::Usage;

// What the chaos binary is asked: to replay one seed, its nodes' logs
// written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invocation {
    Replay { seed: u64 },
}

impl Invocation {
    // The invocation `args` -- the words after the binary's name -- ask for.
    pub fn parse(args: &[String]) -> Result<Self, Usage> {
        match args {
            [command, seed] if command == "replay" => seed
                .parse()
                .map(|seed| Self::Replay { seed })
                .map_err(|_| Usage),
            _ => Err(Usage),
        }
    }
}
