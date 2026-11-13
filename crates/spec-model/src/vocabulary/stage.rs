// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Initial,
    Pinging,
    Collecting,
    Bootstrapped,
    Reconfiguring,
}

impl Stage {
    pub const ALL: [Stage; 5] = [
        Stage::Initial,
        Stage::Pinging,
        Stage::Collecting,
        Stage::Bootstrapped,
        Stage::Reconfiguring,
    ];

    #[must_use]
    pub fn of(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|stage| stage.name() == name)
            .expect("a stage")
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Stage::Initial => "Initial",
            Stage::Pinging => "Pinging",
            Stage::Collecting => "Collecting",
            Stage::Bootstrapped => "Bootstrapped",
            Stage::Reconfiguring => "Reconfiguring",
        }
    }
}
