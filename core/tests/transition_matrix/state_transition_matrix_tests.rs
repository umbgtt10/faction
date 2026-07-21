// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

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
use super::helpers::{
    all_admissible, bootstrapped_admissible, collecting_admissible, initial_admissible,
    join_approved, join_rejected, join_requested, participation, ready,
};

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
    bootstrapped_admissible(),
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
    bootstrapped_admissible(),
)]
#[case::local_completion_redundant_is_rejected(
    Init::CollectingNoReadiness,
    Command::LocalParticipationCompleted,
    &[],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::deadline_missed_from_pinging(
    Init::Fresh,
    Command::DeadlineExpired,
    &[DeadlineMissed { confirmed_count: 0 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::deadline_missed_from_collecting(
    Init::CollectingNoReadiness,
    Command::DeadlineExpired,
    &[DeadlineMissed { confirmed_count: 1 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::bootstrapped_acknowledges_rejoin(
    Init::Bootstrapped,
    participation(1),
    &[AcknowledgeRejoin { peer_id: 1 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::initial_join_requested(
    Init::Initial,
    join_requested(99),
    &[EmitJoinRequest { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    initial_admissible(),
)]
#[case::initial_join_approved_new_member(
    Init::Initial,
    join_approved(99),
    &[MemberAdmitted { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    initial_admissible(),
)]
#[case::initial_join_approved_existing_member(
    Init::Initial,
    join_approved(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    initial_admissible(),
)]
#[case::initial_join_rejected(
    Init::Initial,
    join_rejected(99),
    &[JoinDenied { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    initial_admissible(),
)]
#[case::pinging_join_requested(
    Init::Fresh,
    join_requested(99),
    &[EmitJoinRequest { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::pinging_join_approved_new_member(
    Init::Fresh,
    join_approved(99),
    &[MemberAdmitted { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::pinging_join_approved_existing_member(
    Init::Fresh,
    join_approved(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::pinging_join_rejected(
    Init::Fresh,
    join_rejected(99),
    &[JoinDenied { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::collecting_join_requested(
    Init::CollectingNoReadiness,
    join_requested(99),
    &[EmitJoinRequest { peer_id: 99 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_join_approved_new_member(
    Init::CollectingNoReadiness,
    join_approved(99),
    &[MemberAdmitted { peer_id: 99 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_join_approved_existing_member(
    Init::CollectingNoReadiness,
    join_approved(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_join_rejected(
    Init::CollectingNoReadiness,
    join_rejected(99),
    &[JoinDenied { peer_id: 99 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_acknowledges_member_participation(
    Init::CollectingNoReadiness,
    participation(1),
    &[AcknowledgeRejoin { peer_id: 1 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_ignores_non_member_participation(
    Init::CollectingNoReadiness,
    participation(99),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::bootstrapped_join_requested(
    Init::Bootstrapped,
    join_requested(99),
    &[EmitJoinRequest { peer_id: 99 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_join_approved_new_member(
    Init::Bootstrapped,
    join_approved(99),
    &[MemberAdmitted { peer_id: 99 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_join_approved_existing_member(
    Init::Bootstrapped,
    join_approved(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_join_rejected(
    Init::Bootstrapped,
    join_rejected(99),
    &[JoinDenied { peer_id: 99 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::pinging_join_requested_existing_member(
    Init::Fresh,
    join_requested(1),
    &[EmitJoinRequest { peer_id: 1 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::timed_out_join_requested(
    Init::TimedOut,
    join_requested(99),
    &[EmitJoinRequest { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
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
