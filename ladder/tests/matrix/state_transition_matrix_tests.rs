// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use super::assertions::{verify, Assert};
use super::builder::{build, Init};
use super::helpers::{
    all_admissible, bootstrapped_admissible, collecting_admissible, initial_admissible,
    join_approved, join_rejected, join_requested, participation, ready,
};
use crate::support::matrix_table::MatrixTable;
use crate::support::spec_files::SpecFiles;
use crate::support::spec_writer::SpecWriter;
use alloc::vec;
use alloc::vec::Vec;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::outcome::Outcome;
use faction::outcome::Outcome::*;
use faction::process_result::ProcessResult;
use rstest::rstest;

extern crate alloc;

#[rstest]
#[case::participation_timely_member(
    "participation-accepted",
    Init::Fresh,
    participation(1),
    &[ParticipationAccepted { peer_id: 1 }],
    &[Assert::PingingCount(1), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::participation_non_member(
    "non-member-before-completion",
    Init::Fresh,
    participation(99),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::participation_duplicate(
    "participation-duplicate",
    Init::PingingPeer1Confirmed,
    participation(1),
    &[DuplicateParticipationIgnored { peer_id: 1 }],
    &[Assert::PingingCount(1), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::ready_timely_member(
    "ready-before-completion",
    Init::Fresh,
    ready(1),
    &[ReadyAccepted { peer_id: 1 }],
    &[Assert::CollectingCount(1), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::ready_non_member(
    "non-member-before-completion",
    Init::Fresh,
    ready(99),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::CollectingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::ready_duplicate(
    "ready-duplicate",
    Init::CollectingPeer1Confirmed,
    ready(1),
    &[DuplicateReadyIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(2), Assert::NotExited, Assert::LocalComplete],
    collecting_admissible(),
)]
#[case::ready_timely_triggers_quorum(
    "ready-concluded",
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
    "local-completed",
    Init::Fresh,
    Command::LocalParticipationCompleted,
    &[LocalParticipationCompleted, BroadcastLocalReady],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::local_completion_from_initial_transitions_to_collecting(
    "local-completed",
    Init::Initial,
    Command::LocalParticipationCompleted,
    &[LocalParticipationCompleted, BroadcastLocalReady],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::local_completion_with_preloaded_quorum(
    "local-concluded",
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
    "not-admitted",
    Init::CollectingNoReadiness,
    Command::LocalParticipationCompleted,
    &[],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::deadline_missed_from_pinging(
    "deadline-missed",
    Init::Fresh,
    Command::DeadlineExpired,
    &[DeadlineMissed { confirmed_count: 0 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::deadline_missed_from_collecting(
    "deadline-missed",
    Init::CollectingNoReadiness,
    Command::DeadlineExpired,
    &[DeadlineMissed { confirmed_count: 1 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::bootstrapped_acknowledges_rejoin(
    "rejoin-acknowledged",
    Init::Bootstrapped,
    participation(1),
    &[AcknowledgeRejoin { peer_id: 1 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::initial_join_requested(
    "join-requested",
    Init::Initial,
    join_requested(99),
    &[EmitJoinRequest { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    initial_admissible(),
)]
#[case::initial_join_approved_new_member(
    "join-admitted",
    Init::Initial,
    join_approved(99),
    &[MemberAdmitted { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    initial_admissible(),
)]
#[case::initial_join_approved_existing_member(
    "join-duplicate-member",
    Init::Initial,
    join_approved(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    initial_admissible(),
)]
#[case::initial_join_rejected(
    "join-denied",
    Init::Initial,
    join_rejected(99),
    &[JoinDenied { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    initial_admissible(),
)]
#[case::pinging_join_requested(
    "join-requested",
    Init::Fresh,
    join_requested(99),
    &[EmitJoinRequest { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::pinging_join_approved_new_member(
    "join-admitted",
    Init::Fresh,
    join_approved(99),
    &[MemberAdmitted { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::pinging_join_approved_existing_member(
    "join-duplicate-member",
    Init::Fresh,
    join_approved(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::pinging_join_rejected(
    "join-denied",
    Init::Fresh,
    join_rejected(99),
    &[JoinDenied { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::collecting_join_requested(
    "join-requested",
    Init::CollectingNoReadiness,
    join_requested(99),
    &[EmitJoinRequest { peer_id: 99 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_join_approved_new_member(
    "join-admitted",
    Init::CollectingNoReadiness,
    join_approved(99),
    &[MemberAdmitted { peer_id: 99 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_join_approved_existing_member(
    "join-duplicate-member",
    Init::CollectingNoReadiness,
    join_approved(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_join_rejected(
    "join-denied",
    Init::CollectingNoReadiness,
    join_rejected(99),
    &[JoinDenied { peer_id: 99 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_acknowledges_member_participation(
    "rejoin-acknowledged",
    Init::CollectingNoReadiness,
    participation(1),
    &[AcknowledgeRejoin { peer_id: 1 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_ignores_non_member_participation(
    "non-member",
    Init::CollectingNoReadiness,
    participation(99),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::bootstrapped_join_requested(
    "join-requested",
    Init::Bootstrapped,
    join_requested(99),
    &[EmitJoinRequest { peer_id: 99 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_join_approved_new_member(
    "join-admitted",
    Init::Bootstrapped,
    join_approved(99),
    &[MemberAdmitted { peer_id: 99 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_join_approved_existing_member(
    "join-duplicate-member",
    Init::Bootstrapped,
    join_approved(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_join_rejected(
    "join-denied",
    Init::Bootstrapped,
    join_rejected(99),
    &[JoinDenied { peer_id: 99 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::pinging_join_requested_existing_member(
    "join-requested",
    Init::Fresh,
    join_requested(1),
    &[EmitJoinRequest { peer_id: 1 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::timed_out_join_requested(
    "join-requested",
    Init::TimedOut,
    join_requested(99),
    &[EmitJoinRequest { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    all_admissible(),
)]
#[case::collecting_accepts_readiness_below_the_required_count(
    "ready-accepted",
    Init::CollectingNoReadiness,
    ready(1),
    &[ReadyAccepted { peer_id: 1 }],
    &[Assert::CollectingCount(2), Assert::NotExited, Assert::LocalComplete],
    collecting_admissible(),
)]
#[case::bootstrapped_ignores_non_member_participation(
    "non-member",
    Init::Bootstrapped,
    participation(99),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::CollectingCount(5), Assert::Exited],
    bootstrapped_admissible(),
)]
fn valid_transition(
    #[case] row: &str,
    #[case] init: Init,
    #[case] command: Command,
    #[case] expected_results: &[Outcome],
    #[case] asserts: &[Assert],
    #[case] expected_admissible: Vec<Command>,
) {
    // Arrange
    let table = MatrixTable::new(SpecFiles::matrix());
    let mut faction = build(init);
    let before = table.stage_of(&SpecWriter::result(&faction.process(Command::Probe)));

    // Act
    let result = faction.process(command);

    // Assert
    let (results, admissible, returned_view) = match result.clone() {
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
    let departures = table.departures(
        row,
        &before,
        &SpecWriter::command(&command),
        &SpecWriter::result(&result),
    );
    assert!(departures.is_empty(), "{departures:?}");
    assert_eq!(results.as_slice(), expected_results, "output mismatch");
    assert_eq!(admissible, expected_admissible, "admissible mismatch");
    let probed_view = verify(&mut faction, asserts);
    assert_eq!(
        returned_view, probed_view,
        "returned cluster_view differs from a subsequent probe"
    );
}
