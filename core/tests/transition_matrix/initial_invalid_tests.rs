// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction::process_result::ProcessResult;
use rstest::rstest;

use super::assertions::{verify, Assert};
use super::builder::{build, Init};

#[rstest]
#[case::rejects_deadline_expired(
    Init::Initial,
    Command::DeadlineExpired,
    &[Assert::PingingCount(0), Assert::CollectingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    &[Command::ParticipationObserved { peer_id: 0 }, Command::ReadyObserved { peer_id: 0 }, Command::LocalParticipationCompleted, Command::JoinRequested { peer_id: 0 }, Command::JoinApproved { peer_id: 0 }, Command::JoinRejected { peer_id: 0 }, Command::Probe],
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
