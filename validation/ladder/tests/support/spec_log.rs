// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::spec_writer::SpecWriter;
use faction::command::Command;
use faction::observer::Observer;
use faction::process_result::ProcessResult;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

// The lines a log was written with, shared with the test that reads them.
pub type Lines = Rc<RefCell<Vec<Value>>>;

// An observer that writes every call it is told of as the specification writes
// a log's step -- the command and its result -- in the order it was told them.
pub struct SpecLog {
    lines: Lines,
}

impl SpecLog {
    pub fn new(lines: Lines) -> Self {
        Self { lines }
    }
}

impl Observer for SpecLog {
    fn observe(&mut self, command: &Command, result: &ProcessResult) {
        self.lines
            .borrow_mut()
            .push(SpecWriter::step(command, result));
    }
}
