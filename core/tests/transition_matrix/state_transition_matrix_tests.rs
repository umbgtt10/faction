// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::outcome::Outcome;
use faction::outcome::Outcome::*;
use faction::process_result::ProcessResult;
use rstest::rstest;

use super::assertions::{verify, Assert};
use super::builder::{build, Init};
use super::helpers::{all_admissible, collecting_admissible, participation, probe_only, ready};

#[rstest]
#[case::participation_timely_member(
    Init::Fresh,
    participation(1),
    &[ParticipationAccepted { peer_id: 1 }],
    &[Assert::PingingCount(1), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::participation_non_member(
    Init::Fresh,
    participation(99),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::participation_duplicate(
    Init::PingingPeer1Confirmed,
    participation(1),
    &[DuplicateParticipationIgnored { peer_id: 1 }],
    &[Assert::PingingCount(1), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::ready_timely_member(
    Init::Fresh,
    ready(1),
    &[ReadyAccepted { peer_id: 1 }],
    &[Assert::CollectingCount(1), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::ready_non_member(
    Init::Fresh,
    ready(99),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::CollectingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::ready_duplicate(
    Init::CollectingPeer1Confirmed,
    ready(1),
    &[DuplicateReadyIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(2), Assert::NotExited, Assert::LocalComplete],
    collecting_admissible(),
)]
#[case::ready_timely_triggers_quorum(
    Init::CollectingAlmostQuorum,
    ready(4),
    &[
        ReadyAccepted { peer_id: 4 },
        Concluded { mode: Conclusion::Bootstrapped },
    ],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    probe_only(),
)]
#[case::local_completion_transitions_to_collecting(
    Init::Fresh,
    Command::LocalParticipationCompleted,
    &[LocalParticipationCompleted, BroadcastLocalReady],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::local_completion_from_initial_transitions_to_collecting(
    Init::Initial,
    Command::LocalParticipationCompleted,
    &[LocalParticipationCompleted, BroadcastLocalReady],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::local_completion_with_preloaded_quorum(
    Init::PingingP2Threshold,
    Command::LocalParticipationCompleted,
    &[
        LocalParticipationCompleted,
        BroadcastLocalReady,
        Concluded { mode: Conclusion::Bootstrapped },
    ],
    &[Assert::CollectingCount(5), Assert::LocalComplete, Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    probe_only(),
)]
#[case::local_completion_redundant_is_rejected(
    Init::CollectingNoReadiness,
    Command::LocalParticipationCompleted,
    &[],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::deadline_expired(
    Init::Fresh,
    Command::DeadlineExpired,
    &[Concluded { mode: Conclusion::TimedOut }],
    &[Assert::Exited, Assert::Conclusion(Conclusion::TimedOut)],
    probe_only(),
)]
#[case::deadline_expired_from_collecting(
    Init::CollectingNoReadiness,
    Command::DeadlineExpired,
    &[Concluded { mode: Conclusion::TimedOut }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::Exited, Assert::Conclusion(Conclusion::TimedOut)],
    probe_only(),
)]
fn valid_transition(
    #[case] init: Init,
    #[case] command: Command,
    #[case] expected_results: &[Outcome],
    #[case] asserts: &[Assert],
    #[case] expected_admissible: Vec<Command>,
) {
    // Arrange
    let mut faction = build(init);

    // Act
    let (results, admissible, returned_view) = match faction.process(command) {
        ProcessResult::Accepted {
            outcomes,
            admissible,
            cluster_view,
        } => (outcomes, admissible, cluster_view),
        ProcessResult::Rejected {
            admissible,
            cluster_view,
        } => (vec![], admissible, cluster_view),
        ProcessResult::Probed { .. } => unreachable!(),
    };

    // Assert
    assert_eq!(results.as_slice(), expected_results, "output mismatch");
    assert_eq!(admissible, expected_admissible, "admissible mismatch");
    let probed_view = verify(&mut faction, asserts);
    assert_eq!(
        returned_view, probed_view,
        "returned cluster_view differs from a subsequent probe"
    );
}
