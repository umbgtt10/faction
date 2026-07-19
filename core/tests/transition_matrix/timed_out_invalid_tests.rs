// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::process_result::ProcessResult;
use rstest::rstest;

use super::assertions::{verify, Assert};
use super::builder::{build, Init};
use super::helpers::{participation, ready};

#[rstest]
#[case::rejects_participation_observed(
    Init::TimedOut,
    participation(1),
    &[Assert::PingingCount(0), Assert::CollectingCount(0), Assert::Exited, Assert::Conclusion(Conclusion::TimedOut)],
    &[Command::Probe],
)]
#[case::rejects_participation_observed_non_member(
    Init::TimedOut,
    participation(99),
    &[Assert::PingingCount(0), Assert::CollectingCount(0), Assert::Exited, Assert::Conclusion(Conclusion::TimedOut)],
    &[Command::Probe],
)]
#[case::rejects_ready_observed(
    Init::TimedOut,
    ready(1),
    &[Assert::PingingCount(0), Assert::CollectingCount(0), Assert::Exited, Assert::Conclusion(Conclusion::TimedOut)],
    &[Command::Probe],
)]
#[case::rejects_ready_observed_non_member(
    Init::TimedOut,
    ready(99),
    &[Assert::PingingCount(0), Assert::CollectingCount(0), Assert::Exited, Assert::Conclusion(Conclusion::TimedOut)],
    &[Command::Probe],
)]
#[case::rejects_is_pinging_completedd(
    Init::TimedOut,
    Command::LocalParticipationCompleted,
    &[Assert::PingingCount(0), Assert::CollectingCount(0), Assert::Exited, Assert::Conclusion(Conclusion::TimedOut)],
    &[Command::Probe],
)]
#[case::rejects_deadline_expired(
    Init::TimedOut,
    Command::DeadlineExpired,
    &[Assert::PingingCount(0), Assert::CollectingCount(0), Assert::Exited, Assert::Conclusion(Conclusion::TimedOut)],
    &[Command::Probe],
)]
fn invalid_transition(
    #[case] init: Init,
    #[case] command: Command,
    #[case] asserts: &[Assert],
    #[case] expected_admissible: &[Command],
) {
    // Arrange
    let mut faction = build(init);
    let old_cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    let result = faction.process(command);

    // Assert
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
