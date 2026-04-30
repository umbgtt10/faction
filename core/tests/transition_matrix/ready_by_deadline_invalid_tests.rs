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
    Init::ReadyByDeadline,
    participation(1, TIMELY),
    &[Assert::P1Count(0), Assert::P2Count(0), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Deadline)],
    &[Command::Probe],
)]
#[case::rejects_participation_observed_delayed(
    Init::ReadyByDeadline,
    participation(1, DELAYED),
    &[Assert::P1Count(0), Assert::P2Count(0), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Deadline)],
    &[Command::Probe],
)]
#[case::rejects_participation_observed_non_member(
    Init::ReadyByDeadline,
    participation(99, TIMELY),
    &[Assert::P1Count(0), Assert::P2Count(0), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Deadline)],
    &[Command::Probe],
)]
#[case::rejects_ready_observed(
    Init::ReadyByDeadline,
    ready(1, TIMELY),
    &[Assert::P1Count(0), Assert::P2Count(0), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Deadline)],
    &[Command::Probe],
)]
#[case::rejects_ready_observed_delayed(
    Init::ReadyByDeadline,
    ready(1, DELAYED),
    &[Assert::P1Count(0), Assert::P2Count(0), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Deadline)],
    &[Command::Probe],
)]
#[case::rejects_ready_observed_non_member(
    Init::ReadyByDeadline,
    ready(99, TIMELY),
    &[Assert::P1Count(0), Assert::P2Count(0), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Deadline)],
    &[Command::Probe],
)]
#[case::rejects_local_participation_completed(
    Init::ReadyByDeadline,
    Command::LocalParticipationCompleted,
    &[Assert::P1Count(0), Assert::P2Count(0), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Deadline)],
    &[Command::Probe],
)]
#[case::rejects_deadline_expired(
    Init::ReadyByDeadline,
    Command::DeadlineExpired,
    &[Assert::P1Count(0), Assert::P2Count(0), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Deadline)],
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
        ProcessResult::Probed { snapshot, .. } => snapshot,
        _ => unreachable!(),
    };

    // Act
    let result = m.process(input);

    // Assert
    let (snapshot, admissible) = match result {
        ProcessResult::Rejected {
            snapshot,
            admissible,
        } => (snapshot, admissible),
        other => panic!("expected Rejected, got {other:?}"),
    };
    verify(&mut m, asserts);
    assert_eq!(snapshot, snapshot_before);
    assert_eq!(admissible, expected_admissible);
}
