// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;

const BINARY: &str = "cargo-dry4rust";
const CEILING_EXCEEDED: i32 = 1;

// Duplication in the library's source, over what dry4rust.toml leaves in, at
// the floor the sibling bookkeepers use: zero ceilings against the baseline,
// the duplication already there when the floor came down to it, so a change
// fails on what it adds and passes on what it removes.
pub struct DryGate<'a> {
    runner: &'a dyn CommandRunner,
    path: String,
    baseline: String,
    min_nodes: String,
}

impl<'a> DryGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        path: String,
        baseline: String,
        min_nodes: String,
    ) -> Self {
        Self {
            runner,
            path,
            baseline,
            min_nodes,
        }
    }
}

impl Gate for DryGate<'_> {
    fn label(&self) -> String {
        String::from("Duplication")
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.runner.is_available(BINARY) {
            return Err(GateFailure::NotInstalled {
                binary: String::from(BINARY),
            });
        }

        // Zero ceilings against the baseline: what the baseline records was here
        // before the gate was, and anything beyond it is what a change added. The
        // baseline was recorded at this floor and only matches at this floor.
        let args = vec![
            String::from("dry4rust"),
            String::from("--path"),
            self.path.clone(),
            String::from("--baseline"),
            self.baseline.clone(),
            String::from("--min-nodes"),
            self.min_nodes.clone(),
            String::from("check"),
            String::from("--max-exact"),
            String::from("0"),
            String::from("--max-near"),
            String::from("0"),
        ];

        // 1 is the tool's own "a ceiling was exceeded"; 2 is a configuration or a
        // baseline it refused, which means it measured nothing.
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            Some(CEILING_EXCEEDED) => Err(GateFailure::DuplicationAdded {
                baseline: self.baseline.clone(),
            }),
            code => Err(GateFailure::CouldNotRun { exit_code: code }),
        }
    }
}
