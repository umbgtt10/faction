// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use super::assertions::{verify, Assert};
use super::builder::{build, Init};
use super::helpers::{
    addition_requested, join_approved, ready, reconfiguring_admissible, removal_requested,
};
use crate::support::matrix_table::MatrixTable;
use crate::support::spec_files::SpecFiles;
use crate::support::spec_writer::SpecWriter;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::process_result::ProcessResult;
use rstest::rstest;

#[rstest]
#[case::rejects_ready_observed(
    Init::Reconfiguring,
    ready(1),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::rejects_local_participation_completed(
    Init::Reconfiguring,
    Command::LocalParticipationCompleted,
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::rejects_deadline_expired(
    Init::Reconfiguring,
    Command::DeadlineExpired,
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::rejects_join_approved(
    Init::Reconfiguring,
    join_approved(8),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::rejects_a_second_addition_requested(
    Init::Reconfiguring,
    addition_requested(8),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::rejects_a_removal_requested(
    Init::Reconfiguring,
    removal_requested(1),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::rejects_an_addition_requested_while_a_removal_is_in_flight(
    Init::Removing,
    addition_requested(8),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::rejects_a_second_removal_requested(
    Init::Removing,
    removal_requested(1),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
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
