// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::io::Write;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;

pub struct ProcessChannel {
    input: Mutex<Box<dyn Write + Send>>,
    acks: Mutex<Receiver<()>>,
}

impl ProcessChannel {
    #[must_use]
    pub fn new(input: Box<dyn Write + Send>, acks: Receiver<()>) -> Self {
        Self {
            input: Mutex::new(input),
            acks: Mutex::new(acks),
        }
    }

    pub fn send(&self, line: &str) {
        if let Ok(mut input) = self.input.lock() {
            let _ = writeln!(input, "{line}");
            let _ = input.flush();
        }
        let _ = self.acks.lock().unwrap().recv();
    }
}
