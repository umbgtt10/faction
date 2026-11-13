// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::command::Command;
use faction_spec_model::node_state::NodeState;
use faction_spec_model::reconfiguration::Reconfiguration;
use faction_spec_model::rules::change::Change;
use faction_spec_model::rules::situation::Situation;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::stage::Stage;

const ADMITTED: [CommandName; 2] = [CommandName::ReadyObserved, CommandName::Probe];

fn acknowledged(peer_id: &str) -> Reconfiguration {
    Reconfiguration::AdditionAcknowledged(String::from(peer_id))
}

fn completion() -> Command {
    Command {
        name: CommandName::LocalParticipationCompleted,
        peer_id: None,
        required_count: None,
    }
}

fn naming(name: CommandName, peer_id: &str) -> Command {
    Command {
        name,
        peer_id: Some(String::from(peer_id)),
        required_count: None,
    }
}

fn new_count(required_count: u64) -> Command {
    Command {
        name: CommandName::RequiredCountChanged,
        peer_id: None,
        required_count: Some(required_count),
    }
}

fn readiness_from(peer_id: &str) -> Command {
    Command {
        name: CommandName::ReadyObserved,
        peer_id: Some(String::from(peer_id)),
        required_count: None,
    }
}

fn removal(peer_id: &str) -> Reconfiguration {
    Reconfiguration::RemovalRequested(String::from(peer_id))
}

fn requested(peer_id: &str) -> Reconfiguration {
    Reconfiguration::AdditionRequested(String::from(peer_id))
}

#[test]
fn holds_acknowledgement_not_awaited_from_another_peer_is_true() {
    // Arrange
    let state = NodeStates::reconfiguring(requested("4"));
    let acknowledgement = naming(CommandName::MembershipAcknowledged, "5");

    // Act
    let holds = Situation::AcknowledgementNotAwaited.holds(&acknowledgement, &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_acknowledgement_not_awaited_from_the_requested_peer_is_false() {
    // Arrange
    let state = NodeStates::reconfiguring(requested("4"));
    let acknowledgement = naming(CommandName::MembershipAcknowledged, "4");

    // Act
    let holds = Situation::AcknowledgementNotAwaited.holds(&acknowledgement, &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_acknowledgement_not_awaited_once_acknowledged_is_true() {
    // Arrange
    let state = NodeStates::reconfiguring(acknowledged("4"));
    let acknowledgement = naming(CommandName::MembershipAcknowledged, "4");

    // Act
    let holds = Situation::AcknowledgementNotAwaited.holds(&acknowledgement, &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_acknowledgement_not_awaited_with_a_removal_in_flight_is_true() {
    // Arrange
    let state = NodeStates::reconfiguring(removal("4"));

    // Act
    let holds = Situation::AcknowledgementNotAwaited.holds(
        &naming(CommandName::MembershipAcknowledged, "4"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_always_in_any_situation_is_true() {
    // Arrange
    let state = NodeStates::of(Stage::Initial, &[], &[]);

    // Act
    let holds = Situation::Always.holds(&completion(), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_completion_loses_quorum_for_another_peer_is_false() {
    // Arrange
    let state = NodeState {
        required_count: 3,
        ..NodeStates::reconfiguring(removal("2"))
    };

    // Act
    let holds = Situation::CompletionLosesQuorum.holds(
        &naming(CommandName::RemovalCommitted, "3"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(!holds);
}

#[test]
fn holds_completion_loses_quorum_for_the_peer_of_the_removal_in_flight_the_count_needs_is_true() {
    // Arrange
    let state = NodeState {
        required_count: 3,
        ..NodeStates::reconfiguring(removal("2"))
    };

    // Act
    let holds = Situation::CompletionLosesQuorum.holds(
        &naming(CommandName::RemovalCommitted, "2"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_completion_restores_quorum_for_another_peer_is_false() {
    // Arrange
    let state = NodeState {
        required_count: 4,
        ..NodeStates::reconfiguring(requested("4"))
    };

    // Act
    let holds = Situation::CompletionRestoresQuorum.holds(
        &naming(CommandName::AdditionCommitted, "5"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(!holds);
}

#[test]
fn holds_completion_restores_quorum_for_the_pending_peer_on_a_lost_quorum_is_true() {
    // Arrange
    let state = NodeState {
        required_count: 4,
        ..NodeStates::reconfiguring(requested("4"))
    };

    // Act
    let holds = Situation::CompletionRestoresQuorum.holds(
        &naming(CommandName::AdditionCommitted, "4"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_count_concludes_when_the_collecting_peers_fall_short_of_the_new_count_is_false() {
    // Arrange
    let state = NodeStates::of(Stage::Collecting, &[], &["1"]);

    // Act
    let holds = Situation::CountConcludes.holds(&new_count(3), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_count_concludes_when_the_collecting_peers_reach_the_new_count_is_true() {
    // Arrange
    let state = NodeStates::of(Stage::Collecting, &[], &["1"]);

    // Act
    let holds = Situation::CountConcludes.holds(&new_count(1), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_loses_quorum_when_the_change_takes_the_members_not_suspected_below_the_count_is_true() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let holds = Situation::LosesQuorum(Change::CountSet).holds(&new_count(4), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_loses_quorum_when_the_quorum_stays_held_is_false() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let holds = Situation::LosesQuorum(Change::CountSet).holds(&new_count(3), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_node_concludes_when_the_node_brings_the_count_to_the_required_is_true() {
    // Arrange
    let state = NodeStates::of(Stage::Pinging, &[], &["3"]);

    // Act
    let holds = Situation::NodeConcludes.holds(&completion(), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_node_concludes_with_its_own_readiness_already_counted_is_false() {
    // Arrange
    let state = NodeStates::of(Stage::Pinging, &[], &["1"]);

    // Act
    let holds = Situation::NodeConcludes.holds(&completion(), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_node_is_alone_for_a_node_with_members_is_false() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let holds = Situation::NodeIsAlone.holds(&new_count(1), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_node_is_alone_for_a_node_with_no_other_member_is_true() {
    // Arrange
    let state = NodeState {
        members: NodeStates::peers(&["1"]),
        ..NodeStates::concluded()
    };

    // Act
    let holds = Situation::NodeIsAlone.holds(&new_count(1), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_not_admitted_for_a_command_the_stage_admits_is_false() {
    // Arrange
    let state = NodeStates::of(Stage::Bootstrapped, &[], &[]);

    // Act
    let holds = Situation::NotAdmitted.holds(&readiness_from("2"), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_not_admitted_for_a_command_the_stage_does_not_admit_is_true() {
    // Arrange
    let state = NodeStates::of(Stage::Bootstrapped, &[], &[]);

    // Act
    let holds = Situation::NotAdmitted.holds(&completion(), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_concludes_when_the_count_stays_short_is_false() {
    // Arrange
    let state = NodeStates::of(Stage::Collecting, &[], &[]);

    // Act
    let holds = Situation::PeerConcludes.holds(&readiness_from("2"), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_peer_concludes_when_the_peer_brings_the_count_to_the_required_is_true() {
    // Arrange
    let state = NodeStates::of(Stage::Collecting, &[], &["1"]);

    // Act
    let holds = Situation::PeerConcludes.holds(&readiness_from("2"), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_is_collecting_for_a_collecting_peer_is_true() {
    // Arrange
    let state = NodeStates::of(Stage::Collecting, &[], &["2"]);

    // Act
    let holds = Situation::PeerIsCollecting.holds(&readiness_from("2"), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_is_collecting_for_a_peer_not_collecting_is_false() {
    // Arrange
    let state = NodeStates::of(Stage::Collecting, &["2"], &["3"]);

    // Act
    let holds = Situation::PeerIsCollecting.holds(&readiness_from("2"), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_peer_is_member_for_a_member_is_true() {
    // Arrange
    let state = NodeStates::of(Stage::Pinging, &[], &[]);

    // Act
    let holds = Situation::PeerIsMember.holds(&readiness_from("3"), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_is_member_for_a_stranger_is_false() {
    // Arrange
    let state = NodeStates::of(Stage::Pinging, &[], &[]);

    // Act
    let holds = Situation::PeerIsMember.holds(&readiness_from("7"), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_peer_is_not_member_for_a_member_is_false() {
    // Arrange
    let state = NodeStates::of(Stage::Pinging, &[], &[]);

    // Act
    let holds = Situation::PeerIsNotMember.holds(&readiness_from("3"), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_peer_is_not_member_for_a_stranger_is_true() {
    // Arrange
    let state = NodeStates::of(Stage::Pinging, &[], &[]);

    // Act
    let holds = Situation::PeerIsNotMember.holds(&readiness_from("7"), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_is_not_pending_for_a_removal_s_abort_with_the_peer_s_addition_in_flight_is_true() {
    // Arrange
    let state = NodeStates::reconfiguring(requested("4"));

    // Act
    let holds = Situation::PeerIsNotPending.holds(
        &naming(CommandName::RemovalAborted, "4"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_is_not_pending_for_an_addition_s_abort_with_the_peer_s_removal_in_flight_is_true() {
    // Arrange
    let state = NodeStates::reconfiguring(removal("2"));

    // Act
    let holds = Situation::PeerIsNotPending.holds(
        &naming(CommandName::AdditionAborted, "2"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_is_not_pending_for_another_peer_is_true() {
    // Arrange
    let state = NodeStates::reconfiguring(requested("4"));

    // Act
    let holds = Situation::PeerIsNotPending.holds(
        &naming(CommandName::AdditionAborted, "5"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_is_not_pending_for_the_pending_peer_is_false() {
    // Arrange
    let state = NodeStates::reconfiguring(acknowledged("4"));

    // Act
    let holds = Situation::PeerIsNotPending.holds(
        &naming(CommandName::AdditionAborted, "4"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(!holds);
}

#[test]
fn holds_peer_is_pending_for_an_addition_s_commit_with_the_peer_s_removal_in_flight_is_false() {
    // Arrange
    let state = NodeStates::reconfiguring(removal("2"));

    // Act
    let holds = Situation::PeerIsPending.holds(
        &naming(CommandName::AdditionCommitted, "2"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(!holds);
}

#[test]
fn holds_peer_is_pending_for_another_peer_is_false() {
    // Arrange
    let state = NodeStates::reconfiguring(requested("4"));

    // Act
    let holds = Situation::PeerIsPending.holds(
        &naming(CommandName::AdditionCommitted, "5"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(!holds);
}

#[test]
fn holds_peer_is_pending_for_the_acknowledged_peer_is_true() {
    // Arrange
    let state = NodeStates::reconfiguring(acknowledged("4"));

    // Act
    let holds = Situation::PeerIsPending.holds(
        &naming(CommandName::AdditionCommitted, "4"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_is_pending_for_the_peer_of_the_removal_in_flight_is_true() {
    // Arrange
    let state = NodeStates::reconfiguring(removal("2"));

    // Act
    let holds = [CommandName::RemovalCommitted, CommandName::RemovalAborted]
        .map(|name| Situation::PeerIsPending.holds(&naming(name, "2"), &state, &ADMITTED));

    // Assert
    assert_eq!(holds, [true, true]);
}

#[test]
fn holds_peer_is_pending_for_the_requested_peer_is_true() {
    // Arrange
    let state = NodeStates::reconfiguring(requested("4"));

    // Act
    let holds = Situation::PeerIsPending.holds(
        &naming(CommandName::AdditionCommitted, "4"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_is_pending_with_no_addition_in_flight_is_false() {
    // Arrange
    let state = NodeStates::of(Stage::Bootstrapped, &[], &["1"]);

    // Act
    let holds = Situation::PeerIsPending.holds(
        &naming(CommandName::AdditionCommitted, "4"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(!holds);
}

#[test]
fn holds_peer_is_pinging_for_a_peer_not_pinging_is_false() {
    // Arrange
    let state = NodeStates::of(Stage::Pinging, &["3"], &["2"]);

    // Act
    let holds = Situation::PeerIsPinging.holds(&readiness_from("2"), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_peer_is_pinging_for_a_pinging_peer_is_true() {
    // Arrange
    let state = NodeStates::of(Stage::Pinging, &["2"], &[]);

    // Act
    let holds = Situation::PeerIsPinging.holds(&readiness_from("2"), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_is_self_for_another_member_is_false() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let holds =
        Situation::PeerIsSelf.holds(&naming(CommandName::PeerHeard, "2"), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_peer_is_self_for_the_node_itself_is_true() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let holds =
        Situation::PeerIsSelf.holds(&naming(CommandName::PeerHeard, "1"), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_peer_is_suspected_for_a_member_not_suspected_is_false() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["2"]),
        ..NodeStates::concluded()
    };

    // Act
    let holds =
        Situation::PeerIsSuspected.holds(&naming(CommandName::PeerHeard, "3"), &state, &ADMITTED);

    // Assert
    assert!(!holds);
}

#[test]
fn holds_peer_is_suspected_for_a_suspect_is_true() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["2"]),
        ..NodeStates::concluded()
    };

    // Act
    let holds =
        Situation::PeerIsSuspected.holds(&naming(CommandName::PeerHeard, "2"), &state, &ADMITTED);

    // Assert
    assert!(holds);
}

#[test]
fn holds_quorum_cannot_spare_a_member_once_the_quorum_is_lost_is_true() {
    // Arrange
    let state = NodeState {
        required_count: 3,
        suspected: NodeStates::peers(&["3"]),
        ..NodeStates::concluded()
    };

    // Act
    let holds = Situation::QuorumCannotSpare.holds(
        &naming(CommandName::RemovalRequested, "2"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_quorum_cannot_spare_a_member_the_count_does_not_need_is_false() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let holds = Situation::QuorumCannotSpare.holds(
        &naming(CommandName::RemovalRequested, "2"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(!holds);
}

#[test]
fn holds_quorum_cannot_spare_a_member_the_count_needs_is_true() {
    // Arrange
    let state = NodeState {
        required_count: 3,
        ..NodeStates::concluded()
    };

    // Act
    let holds = Situation::QuorumCannotSpare.holds(
        &naming(CommandName::RemovalRequested, "2"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_quorum_cannot_spare_a_suspect_is_false() {
    // Arrange
    let state = NodeState {
        required_count: 3,
        suspected: NodeStates::peers(&["2"]),
        ..NodeStates::concluded()
    };

    // Act
    let holds = Situation::QuorumCannotSpare.holds(
        &naming(CommandName::RemovalRequested, "2"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(!holds);
}

#[test]
fn holds_quorum_cannot_spare_the_node_itself_the_count_needs_is_true() {
    // Arrange
    let state = NodeState {
        required_count: 3,
        ..NodeStates::concluded()
    };

    // Act
    let holds = Situation::QuorumCannotSpare.holds(
        &naming(CommandName::RemovalRequested, "1"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_restores_quorum_when_the_change_brings_the_members_not_suspected_to_the_count_is_true() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["2", "3"]),
        ..NodeStates::concluded()
    };

    // Act
    let holds = Situation::RestoresQuorum(Change::PeerRevived).holds(
        &naming(CommandName::PeerHeard, "2"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_restores_quorum_when_the_quorum_was_held_is_false() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["2"]),
        ..NodeStates::concluded()
    };

    // Act
    let holds = Situation::RestoresQuorum(Change::PeerRevived).holds(
        &naming(CommandName::PeerHeard, "2"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(!holds);
}

#[test]
fn holds_suspicion_loses_quorum_short_of_the_threshold_is_false() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["2"]),
        suspicion_threshold: 2,
        ..NodeStates::concluded()
    };

    // Act
    let holds = Situation::SuspicionLosesQuorum.holds(
        &naming(CommandName::IndirectProbesMissed, "3"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(!holds);
}

#[test]
fn holds_suspicion_loses_quorum_when_the_threshold_suspects_a_member_the_count_needs_is_true() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["2"]),
        ..NodeStates::concluded()
    };

    // Act
    let holds = Situation::SuspicionLosesQuorum.holds(
        &naming(CommandName::IndirectProbesMissed, "3"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_threshold_reached_by_the_period_the_threshold_counts_is_true() {
    // Arrange
    let state = NodeState {
        suspicion_threshold: 2,
        misses: NodeStates::misses(&[("3", 1)]),
        ..NodeStates::concluded()
    };

    // Act
    let holds = Situation::ThresholdReached.holds(
        &naming(CommandName::IndirectProbesMissed, "3"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(holds);
}

#[test]
fn holds_threshold_reached_short_of_it_is_false() {
    // Arrange
    let state = NodeState {
        suspicion_threshold: 3,
        misses: NodeStates::misses(&[("3", 1)]),
        ..NodeStates::concluded()
    };

    // Act
    let holds = Situation::ThresholdReached.holds(
        &naming(CommandName::IndirectProbesMissed, "3"),
        &state,
        &ADMITTED,
    );

    // Assert
    assert!(!holds);
}
