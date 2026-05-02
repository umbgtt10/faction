// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::command::Command;
use faction::exit_mode::ExitMode;
use faction::outcome::Outcome;
use faction::outcome::Outcome::*;
use faction::process_result::ProcessResult;
use rstest::rstest;

use super::helpers::*;

#[rstest]
#[case::participation_timely_member(
    Init::Fresh,
    participation(1, TIMELY),
    &[ParticipationAccepted { peer_id: 1 }],
    &[Assert::PingingCount(1), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::participation_delayed_member(
    Init::Fresh,
    participation(1, DELAYED),
    &[DelayedParticipationAccepted { peer_id: 1 }],
    &[Assert::PingingCount(1), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::participation_stale_member(
    Init::Fresh,
    participation(1, STALE),
    &[StaleParticipationIgnored { peer_id: 1 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::participation_non_member(
    Init::Fresh,
    participation(99, TIMELY),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::participation_duplicate(
    Init::PingingPeer1Confirmed,
    participation(1, TIMELY),
    &[DuplicateParticipationIgnored { peer_id: 1 }],
    &[Assert::PingingCount(1), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::ready_timely_member(
    Init::Fresh,
    ready(1, TIMELY),
    &[ReadyAccepted { peer_id: 1 }],
    &[Assert::CollectingCount(1), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::ready_delayed_member(
    Init::Fresh,
    ready(1, DELAYED),
    &[DelayedReadyAccepted { peer_id: 1 }],
    &[Assert::CollectingCount(1), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::ready_stale_member(
    Init::Fresh,
    ready(1, STALE),
    &[StaleReadyIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(0), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::ready_non_member(
    Init::Fresh,
    ready(99, TIMELY),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::CollectingCount(0), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::ready_duplicate(
    Init::CollectingPeer1Confirmed,
    ready(1, TIMELY),
    &[DuplicateReadyIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(2), Assert::NotExited, Assert::LocalComplete],
)]
#[case::ready_timely_triggers_quorum(
    Init::CollectingAlmostQuorum,
    ready(4, TIMELY),
    &[
        ReadyAccepted { peer_id: 4 },
        
        Exited { mode: ExitMode::Bootstrapped },
    ],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::ExitMode(ExitMode::Bootstrapped)],
)]
#[case::ready_delayed_triggers_quorum(
    Init::CollectingAlmostQuorum,
    ready(4, DELAYED),
    &[
        DelayedReadyAccepted { peer_id: 4 },
        
        Exited { mode: ExitMode::Bootstrapped },
    ],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::ExitMode(ExitMode::Bootstrapped)],
)]
#[case::local_completion_transitions_to_collecting(
    Init::Fresh,
    Command::LocalParticipationCompleted,
    &[LocalParticipationCompleted, BroadcastLocalReady],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
)]
#[case::local_completion_with_preloaded_quorum(
    Init::PingingP2Threshold,
    Command::LocalParticipationCompleted,
    &[
        LocalParticipationCompleted,
        BroadcastLocalReady,
        
        Exited { mode: ExitMode::Bootstrapped },
    ],
    &[Assert::CollectingCount(5), Assert::LocalComplete, Assert::Exited, Assert::ExitMode(ExitMode::Bootstrapped)],
)]
#[case::local_completion_redundant(
    Init::CollectingNoReadiness,
    Command::LocalParticipationCompleted,
    &[],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
)]
#[case::deadline_expired(
    Init::Fresh,
    Command::DeadlineExpired,
    &[Exited { mode: ExitMode::TimedOut }],
    &[Assert::Exited, Assert::ExitMode(ExitMode::TimedOut)],
)]
#[case::deadline_expired_from_collecting(
    Init::CollectingNoReadiness,
    Command::DeadlineExpired,
    &[Exited { mode: ExitMode::TimedOut }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::Exited, Assert::ExitMode(ExitMode::TimedOut)],
)]
fn valid_transition(
    #[case] init: Init,
    #[case] command: Command,
    #[case] expected_results: &[Outcome],
    #[case] asserts: &[Assert],
) {
    // Arrange
    let mut m = build(init);

    // Act
    let results = match m.process(command) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };

    // Assert
    assert_eq!(results.as_slice(), expected_results, "output mismatch");
    verify(&mut m, asserts);
}
