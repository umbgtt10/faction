// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use super::assertions::{verify, Assert};
use super::builder::{build, Init};
use super::helpers::{
    addition_aborted, addition_committed, addition_requested, indirect_probes_missed,
    join_approved, join_rejected, join_requested, membership_acknowledged, participation,
    peer_heard, probe_missed, ready, removal_aborted, removal_committed, removal_requested,
    removed_admissible, required_count_changed,
};
use crate::support::matrix_table::MatrixTable;
use crate::support::spec_files::SpecFiles;
use crate::support::spec_writer::SpecWriter;
use faction::command::Command;
use faction::process_result::ProcessResult;
use faction::quorum::Quorum;
use rstest::rstest;

#[rstest]
#[case::rejects_participation_observed(
    Init::Removed,
    participation(1),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_ready_observed(
    Init::Removed,
    ready(1),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_local_participation_completed(
    Init::Removed,
    Command::LocalParticipationCompleted,
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_deadline_expired(
    Init::Removed,
    Command::DeadlineExpired,
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_join_requested(
    Init::Removed,
    join_requested(8),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_join_approved(
    Init::Removed,
    join_approved(8),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_join_rejected(
    Init::Removed,
    join_rejected(8),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_addition_requested(
    Init::Removed,
    addition_requested(8),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_membership_acknowledged(
    Init::Removed,
    membership_acknowledged(8),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_addition_committed(
    Init::Removed,
    addition_committed(8),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_addition_aborted(
    Init::Removed,
    addition_aborted(8),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_removal_requested(
    Init::Removed,
    removal_requested(1),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_its_own_removal_committed_again(
    Init::Removed,
    removal_committed(0),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_removal_committed(
    Init::Removed,
    removal_committed(1),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_removal_aborted(
    Init::Removed,
    removal_aborted(1),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_required_count_changed(
    Init::Removed,
    required_count_changed(3),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_probe_due(
    Init::Removed,
    Command::ProbeDue,
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_peer_heard(
    Init::Removed,
    peer_heard(1),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_probe_missed(
    Init::Removed,
    probe_missed(1),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::rejects_indirect_probes_missed(
    Init::Removed,
    indirect_probes_missed(1),
    &[Assert::Removed, Assert::MemberCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
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
