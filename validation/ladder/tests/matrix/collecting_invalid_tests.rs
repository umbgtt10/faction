// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use super::assertions::{verify, Assert};
use super::builder::{build, Init};
use super::helpers::{
    addition_aborted, addition_requested, collecting_admissible, membership_acknowledged,
};
use crate::support::matrix_table::MatrixTable;
use crate::support::spec_files::SpecFiles;
use crate::support::spec_writer::SpecWriter;
use faction::command::Command;
use faction::process_result::ProcessResult;
use rstest::rstest;

#[rstest]
#[case::rejects_is_pinging_completedd(
    Init::CollectingNoReadiness,
    Command::LocalParticipationCompleted,
    &[Assert::PingingCount(0), Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::rejects_is_pinging_completedd_after_ready(
    Init::CollectingPeer1Confirmed,
    Command::LocalParticipationCompleted,
    &[Assert::PingingCount(0), Assert::CollectingCount(2), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::rejects_addition_requested(
    Init::CollectingNoReadiness,
    addition_requested(9),
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::rejects_membership_acknowledged(
    Init::CollectingNoReadiness,
    membership_acknowledged(9),
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::rejects_addition_aborted(
    Init::CollectingNoReadiness,
    addition_aborted(9),
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
fn invalid_transition(
    #[case] init: Init,
    #[case] command: Command,
    #[case] asserts: &[Assert],
    #[case] expected_admissible: Vec<Command>,
) {
    // Arrange
    let mut faction = build(init);
    let table = MatrixTable::new(SpecFiles::matrix());
    let probed = faction.process(Command::Probe);
    let before = table.stage_of(&SpecWriter::result(&probed));
    let old_cluster_view = match probed {
        ProcessResult::Accepted { cluster_view, .. }
        | ProcessResult::Rejected { cluster_view, .. }
        | ProcessResult::Probed { cluster_view, .. } => cluster_view,
    };

    // Act
    let result = faction.process(command);

    // Assert
    let departures = table.departures(
        "not-admitted",
        &before,
        &SpecWriter::command(&command),
        &SpecWriter::result(&result),
    );
    assert!(departures.is_empty(), "{departures:?}");
    let (new_cluster_view, admissible) = match result {
        ProcessResult::Rejected {
            cluster_view,
            admissible,
        } => (cluster_view, admissible),
        other => panic!("expected Rejected, got {other:?}"),
    };
    verify(&mut faction, asserts);
    assert_eq!(new_cluster_view, old_cluster_view);
    assert_eq!(admissible, expected_admissible);
}
