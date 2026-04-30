// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::process_result::ProcessResult;
use rstest::rstest;

use super::helpers::*;

#[rstest]
#[case::rejects_participation_observed(
    Init::Phase2NoReadiness,
    participation(1, TIMELY),
    &[Assert::P1Count(0), Assert::P2Count(1), Assert::LocalComplete, Assert::NotExited],
    &[Command::ReadyObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::DeadlineExpired, Command::GetSnapshot],
)]
#[case::rejects_participation_observed_stale(
    Init::Phase2Peer1Confirmed,
    participation(2, STALE),
    &[Assert::P1Count(0), Assert::P2Count(2), Assert::LocalComplete, Assert::NotExited],
    &[Command::ReadyObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::DeadlineExpired, Command::GetSnapshot],
)]
#[case::rejects_local_participation_completed(
    Init::Phase2NoReadiness,
    Command::LocalParticipationCompleted,
    &[Assert::P1Count(0), Assert::P2Count(1), Assert::LocalComplete, Assert::NotExited],
    &[Command::ReadyObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::DeadlineExpired, Command::GetSnapshot],
)]
#[case::rejects_local_participation_completed_after_ready(
    Init::Phase2Peer1Confirmed,
    Command::LocalParticipationCompleted,
    &[Assert::P1Count(0), Assert::P2Count(2), Assert::LocalComplete, Assert::NotExited],
    &[Command::ReadyObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::DeadlineExpired, Command::GetSnapshot],
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
