// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use super::assertions::{verify, Assert};
use super::builder::{build, Init};
use super::helpers::{
    addition_aborted, bootstrapped_admissible, membership_acknowledged, ready, removal_aborted,
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
    Init::Bootstrapped,
    ready(1),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::rejects_ready_observed_non_member(
    Init::Bootstrapped,
    ready(99),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::rejects_is_pinging_completedd(
    Init::Bootstrapped,
    Command::LocalParticipationCompleted,
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::rejects_deadline_expired(
    Init::Bootstrapped,
    Command::DeadlineExpired,
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::rejects_membership_acknowledged(
    Init::Bootstrapped,
    membership_acknowledged(9),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::rejects_addition_aborted(
    Init::Bootstrapped,
    addition_aborted(9),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::rejects_removal_aborted(
    Init::Bootstrapped,
    removal_aborted(1),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
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
