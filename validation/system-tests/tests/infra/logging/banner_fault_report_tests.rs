// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::SharedOutput;
use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::infra::logging::banner_fault_report::BannerFaultReport;
use faction_system_tests::traits::fault_report::FaultReport;

#[test]
fn report_writes_a_banner_naming_the_fault_the_link_and_the_message() {
    // Arrange
    let output = SharedOutput::default();
    let mut report = BannerFaultReport::new(Box::new(output.clone()));

    // Act
    report.report(
        "LOSS (dropped)",
        0,
        1,
        &TransportMessage::Ping { from: 0, epoch: 0 },
    );

    // Assert
    assert_eq!(
        output.text(),
        "\n###############################################################\n\
         ###  FAULT INJECTED >>> LOSS (dropped)\n\
         ###  peer 0 -> peer 1   Ping { from: 0, epoch: 0 }\n\
         ###############################################################\n\n"
    );
}
