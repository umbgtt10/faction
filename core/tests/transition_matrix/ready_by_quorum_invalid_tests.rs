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
    Init::ReadyByQuorum,
    participation(1, TIMELY),
    &[Assert::P2Count(5), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Quorum)],
    &[Command::GetSnapshot],
)]
#[case::rejects_participation_observed_delayed(
    Init::ReadyByQuorum,
    participation(1, DELAYED),
    &[Assert::P2Count(5), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Quorum)],
    &[Command::GetSnapshot],
)]
#[case::rejects_participation_observed_non_member(
    Init::ReadyByQuorum,
    participation(99, TIMELY),
    &[Assert::P2Count(5), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Quorum)],
    &[Command::GetSnapshot],
)]
#[case::rejects_ready_observed(
    Init::ReadyByQuorum,
    ready(1, TIMELY),
    &[Assert::P2Count(5), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Quorum)],
    &[Command::GetSnapshot],
)]
#[case::rejects_ready_observed_delayed(
    Init::ReadyByQuorum,
    ready(1, DELAYED),
    &[Assert::P2Count(5), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Quorum)],
    &[Command::GetSnapshot],
)]
#[case::rejects_ready_observed_non_member(
    Init::ReadyByQuorum,
    ready(99, TIMELY),
    &[Assert::P2Count(5), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Quorum)],
    &[Command::GetSnapshot],
)]
#[case::rejects_local_participation_completed(
    Init::ReadyByQuorum,
    Command::LocalParticipationCompleted,
    &[Assert::P2Count(5), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Quorum)],
    &[Command::GetSnapshot],
)]
#[case::rejects_deadline_expired(
    Init::ReadyByQuorum,
    Command::DeadlineExpired,
    &[Assert::P2Count(5), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Quorum)],
    &[Command::GetSnapshot],
)]
fn invalid_transition(
    #[case] init: Init,
    #[case] input: Command,
    #[case] asserts: &[Assert],
    #[case] expected_admissible: &[Command],
) {
    // Arrange
    let mut m = build(init);
    let snapshot_before = m.snapshot();

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
    verify(&m, asserts);
    assert_eq!(snapshot, snapshot_before);
    assert_eq!(admissible, expected_admissible);
}
