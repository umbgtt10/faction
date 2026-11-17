// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::io::Result as IoResult;
use std::io::Write;

// Every byte written to both writers: a run's report on the terminal and in
// its log.
pub struct Tee {
    first: Box<dyn Write>,
    second: Box<dyn Write>,
}

impl Tee {
    #[must_use]
    pub fn new(first: Box<dyn Write>, second: Box<dyn Write>) -> Self {
        Self { first, second }
    }
}

impl Write for Tee {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.first.write_all(buf)?;
        self.second.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        self.first.flush()?;
        self.second.flush()
    }
}
