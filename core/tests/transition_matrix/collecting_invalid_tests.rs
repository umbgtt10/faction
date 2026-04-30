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
    &[Assert::PingingCount(0), Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    &[Command::ReadyObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::DeadlineExpired, Command::Probe],
)]
#[case::rejects_participation_observed_stale(
    Init::Phase2Peer1Confirmed,
    participation(2, STALE),
    &[Assert::PingingCount(0), Assert::CollectingCount(2), Assert::LocalComplete, Assert::NotExited],
    &[Command::ReadyObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::DeadlineExpired, Command::Probe],
)]
#[case::rejects_local_participation_completed(
    Init::Phase2NoReadiness,
    Command::LocalParticipationCompleted,
    &[Assert::PingingCount(0), Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    &[Command::ReadyObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::DeadlineExpired, Command::Probe],
)]
#[case::rejects_local_participation_completed_after_ready(
    Init::Phase2Peer1Confirmed,
    Command::LocalParticipationCompleted,
    &[Assert::PingingCount(0), Assert::CollectingCount(2), Assert::LocalComplete, Assert::NotExited],
    &[Command::ReadyObserved { peer_id: 0, freshness: 0, current_marker: 0 }, Command::DeadlineExpired, Command::Probe],
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
