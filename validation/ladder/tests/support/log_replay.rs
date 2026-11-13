// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::replay_report::ReplayReport;
use crate::support::spec_reader::SpecReader;
use crate::support::spec_writer::SpecWriter;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use serde_json::Value;

// A log -- the configuration, then one step per call -- replayed into a node
// created from that configuration: each step's command processed, and the
// result compared with the step's. A node's results carry no number, so a lost
// line is not told from an altered one: either is a departure, at the first
// line the replay answers otherwise.
pub struct LogReplay;

impl LogReplay {
    pub fn replay(log: &[Value]) -> ReplayReport {
        let (configuration, steps) = log
            .split_first()
            .expect("a log opens with its configuration");
        let mut faction = Faction::new(
            SpecReader::configuration(configuration),
            Box::new(NoOpObserver),
        );
        let mut report = ReplayReport {
            lines: steps.len(),
            reconstructed: 0,
            departures: Vec::new(),
        };
        for (index, step) in steps.iter().enumerate() {
            let result =
                SpecWriter::result(&faction.process(SpecReader::command(&step["command"])));
            if result == step["result"] {
                report.reconstructed += 1;
            } else {
                report.departures.push(format!(
                    "line {}: replayed {result}, logged {}",
                    index + 2,
                    step["result"]
                ));
            }
        }
        report
    }
}
