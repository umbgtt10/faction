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
// receipt compared with the step's. Every receipt is numbered, so a line lost
// shows at the first line after it, its logged number past the replay's; an
// altered line departs where it stands.
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
            let receipt =
                SpecWriter::receipt(&faction.process(SpecReader::command(&step["command"])));
            if receipt == step["receipt"] {
                report.reconstructed += 1;
            } else {
                report.departures.push(format!(
                    "line {}: replayed {receipt}, logged {}",
                    index + 2,
                    step["receipt"]
                ));
            }
        }
        report
    }
}
