// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::process_result::ProcessResult;
use rstest::rstest;

use super::helpers::*;

#[rstest]
#[case::rejects_participation_observed(
    Init::CollectingNoReadiness,
    participation(1),
    &[Assert::PingingCount(0), Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    &[Command::ReadyObserved { peer_id: 0 }, Command::DeadlineExpired, Command::Probe],
)]
#[case::rejects_is_pinging_completedd(
    Init::CollectingNoReadiness,
    Command::LocalParticipationCompleted,
    &[Assert::PingingCount(0), Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    &[Command::ReadyObserved { peer_id: 0 }, Command::DeadlineExpired, Command::Probe],
)]
#[case::rejects_is_pinging_completedd_after_ready(
    Init::CollectingPeer1Confirmed,
    Command::LocalParticipationCompleted,
    &[Assert::PingingCount(0), Assert::CollectingCount(2), Assert::LocalComplete, Assert::NotExited],
    &[Command::ReadyObserved { peer_id: 0 }, Command::DeadlineExpired, Command::Probe],
)]
fn invalid_transition(
    #[case] init: Init,
    #[case] command: Command,
    #[case] asserts: &[Assert],
    #[case] expected_admissible: &[Command],
) {
    // Arrange
    let mut m = build(init);
    let old_cluster_view = match m.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    let result = m.process(command);

    // Assert
    let (new_cluster_view, admissible) = match result {
        ProcessResult::Rejected {
            cluster_view,
            admissible,
        } => (cluster_view, admissible),
        other => panic!("expected Rejected, got {other:?}"),
    };
    verify(&mut m, asserts);
    assert_eq!(new_cluster_view, old_cluster_view);
    assert_eq!(admissible, expected_admissible);
}
