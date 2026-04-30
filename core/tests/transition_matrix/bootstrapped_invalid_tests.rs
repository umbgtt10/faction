// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::process_result::ProcessResult;
use faction::readiness_exit_mode::ReadinessExitMode;
use rstest::rstest;

use super::helpers::*;

#[rstest]
#[case::rejects_participation_observed(
    Init::Bootstrapped,
    participation(1, TIMELY),
    &[Assert::CollectingCount(4), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Bootstrapped)],
    &[Command::Probe],
)]
#[case::rejects_participation_observed_delayed(
    Init::Bootstrapped,
    participation(1, DELAYED),
    &[Assert::CollectingCount(4), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Bootstrapped)],
    &[Command::Probe],
)]
#[case::rejects_participation_observed_non_member(
    Init::Bootstrapped,
    participation(99, TIMELY),
    &[Assert::CollectingCount(4), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Bootstrapped)],
    &[Command::Probe],
)]
#[case::rejects_ready_observed(
    Init::Bootstrapped,
    ready(1, TIMELY),
    &[Assert::CollectingCount(4), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Bootstrapped)],
    &[Command::Probe],
)]
#[case::rejects_ready_observed_delayed(
    Init::Bootstrapped,
    ready(1, DELAYED),
    &[Assert::CollectingCount(4), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Bootstrapped)],
    &[Command::Probe],
)]
#[case::rejects_ready_observed_non_member(
    Init::Bootstrapped,
    ready(99, TIMELY),
    &[Assert::CollectingCount(4), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Bootstrapped)],
    &[Command::Probe],
)]
#[case::rejects_is_pinging_completedd(
    Init::Bootstrapped,
    Command::LocalParticipationCompleted,
    &[Assert::CollectingCount(4), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Bootstrapped)],
    &[Command::Probe],
)]
#[case::rejects_deadline_expired(
    Init::Bootstrapped,
    Command::DeadlineExpired,
    &[Assert::CollectingCount(4), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Bootstrapped)],
    &[Command::Probe],
)]
fn invalid_transition(
    #[case] init: Init,
    #[case] input: Command,
    #[case] asserts: &[Assert],
    #[case] expected_admissible: &[Command],
) {
    // Arrange
    let mut m = build(init);
    let snapshot_before = match m.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    let result = m.process(input);

    // Assert
    let (cluster_view, admissible) = match result {
        ProcessResult::Rejected {
            cluster_view,
            admissible,
        } => (cluster_view, admissible),
        other => panic!("expected Rejected, got {other:?}"),
    };
    verify(&mut m, asserts);
    assert_eq!(cluster_view, snapshot_before);
    assert_eq!(admissible, expected_admissible);
}
