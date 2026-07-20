// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::process_result::ProcessResult;
use rstest::rstest;

use super::assertions::{verify, Assert};
use super::builder::{build, Init};
use super::helpers::ready;

#[rstest]
#[case::rejects_ready_observed(
    Init::Bootstrapped,
    ready(1),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    &[Command::ParticipationObserved { peer_id: 0 }, Command::Probe],
)]
#[case::rejects_ready_observed_non_member(
    Init::Bootstrapped,
    ready(99),
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    &[Command::ParticipationObserved { peer_id: 0 }, Command::Probe],
)]
#[case::rejects_is_pinging_completedd(
    Init::Bootstrapped,
    Command::LocalParticipationCompleted,
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    &[Command::ParticipationObserved { peer_id: 0 }, Command::Probe],
)]
#[case::rejects_deadline_expired(
    Init::Bootstrapped,
    Command::DeadlineExpired,
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    &[Command::ParticipationObserved { peer_id: 0 }, Command::Probe],
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
