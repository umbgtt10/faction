// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::process_result::ProcessResult;
use rstest::rstest;

use super::helpers::*;

#[rstest]
#[case::rejects_is_pinging_completedd(
    Init::Initial,
    Command::LocalParticipationCompleted,
    &[Assert::PingingCount(0), Assert::CollectingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    &[Command::ParticipationObserved { peer_id: 0 }, Command::ReadyObserved { peer_id: 0 }, Command::Probe],
)]
#[case::rejects_deadline_expired(
    Init::Initial,
    Command::DeadlineExpired,
    &[Assert::PingingCount(0), Assert::CollectingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    &[Command::ParticipationObserved { peer_id: 0 }, Command::ReadyObserved { peer_id: 0 }, Command::Probe],
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
