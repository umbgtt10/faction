// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use super::assertions::{verify, Assert};
use super::builder::{build, Init};
use super::helpers::{
    addition_aborted, addition_committed, addition_requested, bootstrapped_admissible,
    collecting_admissible, indirect_probes_missed, initial_admissible, join_approved,
    join_rejected, join_requested, membership_acknowledged, participation, peer_heard,
    pinging_admissible, probe_missed, ready, reconfiguring_admissible, removal_aborted,
    removal_committed, removal_requested, removed_admissible, required_count_changed,
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
use faction::quorum::Quorum;
use rstest::rstest;

extern crate alloc;

#[rstest]
#[case::participation_timely_member(
    "participation-accepted",
    Init::Fresh,
    participation(1),
    &[ParticipationAccepted { peer_id: 1 }],
    &[Assert::PingingCount(1), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
)]
#[case::participation_non_member(
    "non-member-before-completion",
    Init::Fresh,
    participation(99),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
)]
#[case::participation_duplicate(
    "participation-duplicate",
    Init::PingingPeer1Confirmed,
    participation(1),
    &[DuplicateParticipationIgnored { peer_id: 1 }],
    &[Assert::PingingCount(1), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
)]
#[case::ready_timely_member(
    "ready-before-completion",
    Init::Fresh,
    ready(1),
    &[ReadyAccepted { peer_id: 1 }],
    &[Assert::CollectingCount(1), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
)]
#[case::ready_non_member(
    "non-member-before-completion",
    Init::Fresh,
    ready(99),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::CollectingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
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
    pinging_admissible(),
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
    pinging_admissible(),
)]
#[case::pinging_join_approved_new_member(
    "join-admitted",
    Init::Fresh,
    join_approved(99),
    &[MemberAdmitted { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
)]
#[case::pinging_join_approved_existing_member(
    "join-duplicate-member",
    Init::Fresh,
    join_approved(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
)]
#[case::pinging_join_rejected(
    "join-denied",
    Init::Fresh,
    join_rejected(99),
    &[JoinDenied { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
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
    pinging_admissible(),
)]
#[case::timed_out_join_requested(
    "join-requested",
    Init::TimedOut,
    join_requested(99),
    &[EmitJoinRequest { peer_id: 99 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
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
#[case::initial_addition_committed_new_member(
    "addition-committed",
    Init::Initial,
    addition_committed(9),
    &[MemberAdmitted { peer_id: 9 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    initial_admissible(),
)]
#[case::initial_addition_committed_existing_member(
    "commit-member",
    Init::Initial,
    addition_committed(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    initial_admissible(),
)]
#[case::initial_required_count_changed(
    "count-set",
    Init::Initial,
    required_count_changed(3),
    &[RequiredCountSet { required_count: 3 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    initial_admissible(),
)]
#[case::pinging_addition_committed_new_member(
    "addition-committed",
    Init::Fresh,
    addition_committed(9),
    &[MemberAdmitted { peer_id: 9 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
)]
#[case::pinging_addition_committed_existing_member(
    "commit-member",
    Init::Fresh,
    addition_committed(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
)]
#[case::pinging_required_count_changed(
    "count-set",
    Init::Fresh,
    required_count_changed(3),
    &[RequiredCountSet { required_count: 3 }],
    &[Assert::PingingCount(0), Assert::NotExited, Assert::NotLocalComplete],
    pinging_admissible(),
)]
#[case::collecting_addition_committed_new_member(
    "addition-committed",
    Init::CollectingNoReadiness,
    addition_committed(9),
    &[MemberAdmitted { peer_id: 9 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_addition_committed_existing_member(
    "commit-member",
    Init::CollectingNoReadiness,
    addition_committed(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_required_count_changed_short(
    "count-set",
    Init::CollectingNoReadiness,
    required_count_changed(3),
    &[RequiredCountSet { required_count: 3 }],
    &[Assert::CollectingCount(1), Assert::LocalComplete, Assert::NotExited],
    collecting_admissible(),
)]
#[case::collecting_required_count_changed_reached_concludes(
    "count-concluded",
    Init::CollectingNoReadiness,
    required_count_changed(1),
    &[RequiredCountSet { required_count: 1 }, Concluded { mode: Conclusion::Bootstrapped }],
    &[Assert::CollectingCount(1), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_addition_requested_newcomer(
    "addition-requested",
    Init::Bootstrapped,
    addition_requested(9),
    &[SendMembership { peer_id: 9 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::bootstrapped_addition_requested_member(
    "addition-member",
    Init::Bootstrapped,
    addition_requested(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_addition_committed_new_member(
    "addition-committed",
    Init::Bootstrapped,
    addition_committed(9),
    &[MemberAdmitted { peer_id: 9 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_addition_committed_existing_member(
    "commit-member",
    Init::Bootstrapped,
    addition_committed(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_required_count_changed(
    "count-set",
    Init::Bootstrapped,
    required_count_changed(3),
    &[RequiredCountSet { required_count: 3 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::reconfiguring_acknowledges_member_participation(
    "rejoin-acknowledged",
    Init::Reconfiguring,
    participation(1),
    &[AcknowledgeRejoin { peer_id: 1 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_ignores_the_newcomer_s_participation(
    "non-member",
    Init::Reconfiguring,
    participation(9),
    &[NonMemberIgnored { peer_id: 9 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_join_requested(
    "join-requested",
    Init::Reconfiguring,
    join_requested(8),
    &[EmitJoinRequest { peer_id: 8 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_join_rejected(
    "join-denied",
    Init::Reconfiguring,
    join_rejected(8),
    &[JoinDenied { peer_id: 8 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_acknowledgement_accepted(
    "acknowledgement-accepted",
    Init::Reconfiguring,
    membership_acknowledged(9),
    &[ProposeAddition { peer_id: 9 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_acknowledgement_from_another_peer(
    "acknowledgement-ignored",
    Init::Reconfiguring,
    membership_acknowledged(8),
    &[AcknowledgementIgnored { peer_id: 8 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_acknowledgement_repeated(
    "acknowledgement-ignored",
    Init::ReconfiguringAcknowledged,
    membership_acknowledged(9),
    &[AcknowledgementIgnored { peer_id: 9 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_addition_completed(
    "addition-completed",
    Init::ReconfiguringAcknowledged,
    addition_committed(9),
    &[MemberAdmitted { peer_id: 9 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::reconfiguring_addition_committed_of_another_peer(
    "addition-committed",
    Init::Reconfiguring,
    addition_committed(8),
    &[MemberAdmitted { peer_id: 8 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_addition_committed_of_a_member(
    "commit-member",
    Init::Reconfiguring,
    addition_committed(1),
    &[DuplicateMemberIgnored { peer_id: 1 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_addition_aborted(
    "addition-aborted",
    Init::Reconfiguring,
    addition_aborted(9),
    &[AdditionWithdrawn { peer_id: 9 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    bootstrapped_admissible(),
)]
#[case::reconfiguring_abort_of_another_peer(
    "abort-ignored",
    Init::Reconfiguring,
    addition_aborted(8),
    &[AbortIgnored { peer_id: 8 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_required_count_changed(
    "count-set",
    Init::Reconfiguring,
    required_count_changed(3),
    &[RequiredCountSet { required_count: 3 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Conclusion(Conclusion::Bootstrapped)],
    reconfiguring_admissible(),
)]
#[case::bootstrapped_probe_due(
    "probe-due",
    Init::Bootstrapped,
    Command::ProbeDue,
    &[SendProbe { peer_id: 1 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Quorum(Quorum::Held)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_peer_heard(
    "alive-noted",
    Init::Bootstrapped,
    peer_heard(1),
    &[AliveNoted { peer_id: 1 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Quorum(Quorum::Held)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_peer_heard_from_itself(
    "liveness-self",
    Init::Bootstrapped,
    peer_heard(0),
    &[SelfIgnored { peer_id: 0 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Quorum(Quorum::Held)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_probe_missed_of_a_non_member(
    "liveness-non-member",
    Init::Bootstrapped,
    probe_missed(8),
    &[NonMemberIgnored { peer_id: 8 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Quorum(Quorum::Held)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_probe_missed(
    "probe-missed",
    Init::Bootstrapped,
    probe_missed(2),
    &[SendIndirectProbes {
        peer_id: 2,
        via: vec![3, 4, 1],
    }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Quorum(Quorum::Held)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_indirect_probes_missed(
    "suspicion-loses-quorum",
    Init::Bootstrapped,
    indirect_probes_missed(1),
    &[PeerSuspected { peer_id: 1 }, QuorumLost],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::SuspectedCount(1), Assert::Quorum(Quorum::Lost)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_required_count_raised_past_the_members(
    "count-loses-quorum",
    Init::Bootstrapped,
    required_count_changed(6),
    &[RequiredCountSet { required_count: 6 }, QuorumLost],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Quorum(Quorum::Lost)],
    bootstrapped_admissible(),
)]
#[case::reconfiguring_probe_due(
    "probe-due",
    Init::Reconfiguring,
    Command::ProbeDue,
    &[SendProbe { peer_id: 1 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Quorum(Quorum::Held)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_peer_heard(
    "alive-noted",
    Init::Reconfiguring,
    peer_heard(2),
    &[AliveNoted { peer_id: 2 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Quorum(Quorum::Held)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_probe_missed_of_the_newcomer(
    "liveness-non-member",
    Init::Reconfiguring,
    probe_missed(9),
    &[NonMemberIgnored { peer_id: 9 }],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::Quorum(Quorum::Held)],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_indirect_probes_missed(
    "suspicion-loses-quorum",
    Init::Reconfiguring,
    indirect_probes_missed(3),
    &[PeerSuspected { peer_id: 3 }, QuorumLost],
    &[Assert::CollectingCount(5), Assert::Exited, Assert::SuspectedCount(1), Assert::Quorum(Quorum::Lost)],
    reconfiguring_admissible(),
)]
#[case::bootstrapped_removal_requested_of_a_non_member(
    "removal-non-member",
    Init::Bootstrapped,
    removal_requested(8),
    &[NonMemberIgnored { peer_id: 8 }],
    &[Assert::MemberCount(5), Assert::Exited],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_removal_refused_for_the_quorum(
    "removal-refused",
    Init::Bootstrapped,
    removal_requested(1),
    &[RemovalRefused { peer_id: 1 }],
    &[Assert::MemberCount(5), Assert::Quorum(Quorum::Held)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_own_removal_refused_for_the_quorum(
    "removal-refused",
    Init::Bootstrapped,
    removal_requested(0),
    &[RemovalRefused { peer_id: 0 }],
    &[Assert::MemberCount(5), Assert::Exited],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_removal_of_a_suspect_requested(
    "removal-requested",
    Init::Suspecting,
    removal_requested(4),
    &[ProposeRemoval { peer_id: 4 }],
    &[Assert::MemberCount(5), Assert::Exited, Assert::SuspectedCount(1)],
    reconfiguring_admissible(),
)]
#[case::removing_removal_aborted(
    "removal-aborted",
    Init::Removing,
    removal_aborted(4),
    &[RemovalWithdrawn { peer_id: 4 }],
    &[Assert::MemberCount(5), Assert::Exited],
    bootstrapped_admissible(),
)]
#[case::removing_abort_of_another_peer(
    "abort-ignored",
    Init::Removing,
    removal_aborted(3),
    &[AbortIgnored { peer_id: 3 }],
    &[Assert::MemberCount(5), Assert::Exited],
    reconfiguring_admissible(),
)]
#[case::removing_abort_of_an_addition(
    "abort-ignored",
    Init::Removing,
    addition_aborted(4),
    &[AbortIgnored { peer_id: 4 }],
    &[Assert::MemberCount(5), Assert::Exited],
    reconfiguring_admissible(),
)]
#[case::reconfiguring_abort_of_a_removal(
    "abort-ignored",
    Init::Reconfiguring,
    removal_aborted(9),
    &[AbortIgnored { peer_id: 9 }],
    &[Assert::MemberCount(5), Assert::Exited],
    reconfiguring_admissible(),
)]
#[case::removing_acknowledgement_ignored(
    "acknowledgement-ignored",
    Init::Removing,
    membership_acknowledged(4),
    &[AcknowledgementIgnored { peer_id: 4 }],
    &[Assert::MemberCount(5), Assert::Exited],
    reconfiguring_admissible(),
)]
#[case::initial_removal_committed_of_a_non_member(
    "commit-non-member",
    Init::Initial,
    removal_committed(8),
    &[NonMemberIgnored { peer_id: 8 }],
    &[Assert::MemberCount(5), Assert::NotExited],
    initial_admissible(),
)]
#[case::collecting_removal_committed_of_a_non_member(
    "commit-non-member",
    Init::CollectingNoReadiness,
    removal_committed(8),
    &[NonMemberIgnored { peer_id: 8 }],
    &[Assert::MemberCount(5), Assert::LocalComplete],
    collecting_admissible(),
)]
#[case::initial_own_removal(
    "node-removed",
    Init::Initial,
    removal_committed(0),
    &[MemberRemoved { peer_id: 0 }],
    &[Assert::Removed, Assert::MemberCount(4), Assert::NotLocalComplete],
    removed_admissible(),
)]
#[case::bootstrapped_own_removal(
    "node-removed",
    Init::Bootstrapped,
    removal_committed(0),
    &[MemberRemoved { peer_id: 0 }],
    &[Assert::Removed, Assert::MemberCount(4), Assert::LocalComplete, Assert::CollectingCount(4), Assert::Quorum(Quorum::NotTracked)],
    removed_admissible(),
)]
#[case::removing_own_removal(
    "node-removed",
    Init::Removing,
    removal_committed(0),
    &[MemberRemoved { peer_id: 0 }],
    &[Assert::Removed, Assert::SuspectedCount(0)],
    removed_admissible(),
)]
#[case::removing_removal_completion_loses_the_quorum(
    "removal-completion-loses-quorum",
    Init::RemovingRevived,
    removal_committed(4),
    &[MemberRemoved { peer_id: 4 }, QuorumLost],
    &[Assert::MemberCount(4), Assert::Quorum(Quorum::Lost)],
    bootstrapped_admissible(),
)]
#[case::removing_removal_completed(
    "removal-completed",
    Init::Removing,
    removal_committed(4),
    &[MemberRemoved { peer_id: 4 }],
    &[Assert::MemberCount(4), Assert::SuspectedCount(0), Assert::Quorum(Quorum::Lost)],
    bootstrapped_admissible(),
)]
#[case::bootstrapped_removal_loses_the_quorum(
    "removal-loses-quorum",
    Init::Bootstrapped,
    removal_committed(1),
    &[MemberRemoved { peer_id: 1 }, QuorumLost],
    &[Assert::MemberCount(4), Assert::CollectingCount(4), Assert::Quorum(Quorum::Lost)],
    bootstrapped_admissible(),
)]
#[case::reconfiguring_removal_loses_the_quorum(
    "removal-loses-quorum",
    Init::Reconfiguring,
    removal_committed(1),
    &[MemberRemoved { peer_id: 1 }, QuorumLost],
    &[Assert::MemberCount(4), Assert::Quorum(Quorum::Lost)],
    reconfiguring_admissible(),
)]
#[case::pinging_removal_committed(
    "removal-committed",
    Init::PingingPeer1Confirmed,
    removal_committed(1),
    &[MemberRemoved { peer_id: 1 }],
    &[Assert::MemberCount(4), Assert::PingingCount(0), Assert::NotExited],
    pinging_admissible(),
)]
#[case::collecting_removal_committed(
    "removal-committed",
    Init::CollectingPeer1Confirmed,
    removal_committed(1),
    &[MemberRemoved { peer_id: 1 }],
    &[Assert::MemberCount(4), Assert::CollectingCount(1), Assert::LocalComplete],
    collecting_admissible(),
)]
#[case::suspecting_removal_of_the_suspect_committed(
    "removal-committed",
    Init::Suspecting,
    removal_committed(4),
    &[MemberRemoved { peer_id: 4 }],
    &[Assert::MemberCount(4), Assert::SuspectedCount(0), Assert::Quorum(Quorum::Lost)],
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
