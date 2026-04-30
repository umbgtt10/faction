// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::process_result::ProcessResult;
use rstest::rstest;

use super::helpers::*;

#[rstest]
#[case::rejects_local_participation_completed(
    Init::Initial,
    Command::LocalParticipationCompleted,
    &[Assert::P1Count(0), Assert::P2Count(0), Assert::NotExited, Assert::NotLocalComplete],
    &[Command::ParticipationObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::ReadyObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::GetSnapshot],
)]
#[case::rejects_deadline_expired(
    Init::Initial,
    Command::DeadlineExpired,
    &[Assert::P1Count(0), Assert::P2Count(0), Assert::NotExited, Assert::NotLocalComplete],
    &[Command::ParticipationObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::ReadyObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::GetSnapshot],
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
