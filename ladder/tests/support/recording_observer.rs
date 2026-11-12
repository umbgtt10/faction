// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction::observer::Observer;
use faction::process_result::ProcessResult;
use std::cell::RefCell;
use std::rc::Rc;

// The calls an observer was told of, each with its result, shared with the
// test that reads them.
pub type Calls = Rc<RefCell<Vec<(Command, ProcessResult)>>>;

// An observer that keeps every call it is told of, in a list its test holds
// too, so what a node reported can be read after the observer was handed to it.
pub struct RecordingObserver {
    calls: Calls,
}

impl RecordingObserver {
    pub fn new(calls: Calls) -> Self {
        Self { calls }
    }
}

impl Observer for RecordingObserver {
    fn observe(&mut self, command: &Command, result: &ProcessResult) {
        self.calls.borrow_mut().push((*command, result.clone()));
    }
}
