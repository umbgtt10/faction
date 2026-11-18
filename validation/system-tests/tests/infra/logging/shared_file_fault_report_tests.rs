// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::traits::wall_clock_tests::FAKE_MOMENT;
use crate::traits::wall_clock_tests::FakeWallClock;
use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::infra::logging::shared_file_fault_report::SharedFileFaultReport;
use faction_system_tests::infra::logging::shared_writer::SharedWriter;
use faction_system_tests::traits::fault_report::FaultReport;
use std::env::temp_dir;
use std::fs::read_to_string;
use std::fs::remove_file;
use std::process::id;
use std::sync::Arc;

#[test]
fn report_writes_one_line_of_the_node_s_log() {
    // Arrange
    let path = temp_dir().join(format!("shared-file-fault-report-tests-{}.jsonl", id()));
    let _ = remove_file(&path);
    let mut report = SharedFileFaultReport::new(
        SharedWriter::open(&path),
        Arc::new(FakeWallClock::at(FAKE_MOMENT)),
    );

    // Act
    report.report(
        "LOSS (dropped)",
        0,
        1,
        &TransportMessage::Ping { from: 0, epoch: 0 },
    );

    // Assert
    let log = read_to_string(&path).unwrap();
    remove_file(&path).ok();
    assert_eq!(
        log,
        "{\"timestamp\":\"2026-11-12T12:30:45.000Z\",\"peer_id\":0,\"event\":\"fault\",\"fault\":\"LOSS (dropped)\",\"to\":1,\"message\":\"Ping { from: 0, epoch: 0 }\"}\n"
    );
}
