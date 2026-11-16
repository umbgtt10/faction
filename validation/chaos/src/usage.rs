// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

// What the chaos binary answers a command it does not know.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usage;

impl Display for Usage {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "usage: faction-chaos replay <seed>")
    }
}

impl Error for Usage {}
