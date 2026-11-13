// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::command::Command;
use faction_spec_model::node_state::NodeState;
use faction_spec_model::reconfiguration::Reconfiguration;
use faction_spec_model::rules::change::Change;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::stage::Stage;

fn command(name: CommandName, peer_id: Option<&str>) -> Command {
    Command {
        name,
        peer_id: peer_id.map(String::from),
        required_count: None,
    }
}

fn state() -> NodeState {
    NodeStates::of(Stage::Pinging, &["3"], &["2"])
}

#[test]
fn applied_addition_acknowledged_acknowledges_the_peer_s_addition() {
    // Arrange
    let acknowledgement = command(CommandName::MembershipAcknowledged, Some("4"));
    let requested =
        NodeStates::reconfiguring(Reconfiguration::AdditionRequested(String::from("4")));

    // Act
    let changed = Change::AdditionAcknowledged.applied(&acknowledgement, &requested);

    // Assert
    assert_eq!(
        changed,
        NodeStates::reconfiguring(Reconfiguration::AdditionAcknowledged(String::from("4")))
    );
}

#[test]
fn applied_addition_completes_makes_the_peer_the_last_member_and_ends_the_addition() {
    // Arrange
    let commit = command(CommandName::AdditionCommitted, Some("4"));
    let acknowledged =
        NodeStates::reconfiguring(Reconfiguration::AdditionAcknowledged(String::from("4")));

    // Act
    let changed = Change::AdditionCompletes.applied(&commit, &acknowledged);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            members: NodeStates::peers(&["1", "2", "3", "4"]),
            ..NodeStates::reconfiguring(Reconfiguration::None)
        }
    );
}

#[test]
fn applied_addition_pends_requests_the_addition_of_the_peer() {
    // Arrange
    let request = command(CommandName::AdditionRequested, Some("4"));
    let concluded = NodeStates::of(Stage::Bootstrapped, &[], &["1"]);

    // Act
    let changed = Change::AdditionPends.applied(&request, &concluded);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            reconfiguration: Reconfiguration::AdditionRequested(String::from("4")),
            ..concluded
        }
    );
}

#[test]
fn applied_addition_withdrawn_leaves_no_addition_in_flight() {
    // Arrange
    let abort = command(CommandName::AdditionAborted, Some("4"));
    let acknowledged =
        NodeStates::reconfiguring(Reconfiguration::AdditionAcknowledged(String::from("4")));

    // Act
    let changed = Change::AdditionWithdrawn.applied(&abort, &acknowledged);

    // Assert
    assert_eq!(changed, NodeStates::reconfiguring(Reconfiguration::None));
}

#[test]
fn applied_count_set_makes_the_command_s_count_the_count_in_effect() {
    // Arrange
    let new_count = Command {
        name: CommandName::RequiredCountChanged,
        peer_id: None,
        required_count: Some(5),
    };

    // Act
    let changed = Change::CountSet.applied(&new_count, &state());

    // Assert
    assert_eq!(
        changed,
        NodeState {
            required_count: 5,
            ..state()
        }
    );
}

#[test]
fn applied_miss_counted_counts_one_more_missed_period() {
    // Arrange
    let state = NodeState {
        misses: NodeStates::misses(&[("2", 1)]),
        ..NodeStates::concluded()
    };

    // Act
    let changed = Change::MissCounted.applied(
        &command(CommandName::IndirectProbesMissed, Some("2")),
        &state,
    );

    // Assert
    assert_eq!(
        changed,
        NodeState {
            misses: NodeStates::misses(&[("2", 2)]),
            ..NodeStates::concluded()
        }
    );
}

#[test]
fn applied_misses_cleared_forgets_the_peer_s_missed_periods() {
    // Arrange
    let state = NodeState {
        misses: NodeStates::misses(&[("2", 1), ("3", 1)]),
        ..NodeStates::concluded()
    };

    // Act
    let changed =
        Change::MissesCleared.applied(&command(CommandName::PeerHeard, Some("2")), &state);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            misses: NodeStates::misses(&[("3", 1)]),
            ..NodeStates::concluded()
        }
    );
}

#[test]
fn applied_misses_deadline_records_the_missed_deadline_alone() {
    // Arrange
    let expiry = command(CommandName::DeadlineExpired, None);

    // Act
    let changed = Change::MissesDeadline.applied(&expiry, &state());

    // Assert
    assert_eq!(
        changed,
        NodeState {
            deadline_missed: true,
            ..state()
        }
    );
}

#[test]
fn applied_node_collects_makes_the_node_the_last_collecting_peer() {
    // Arrange
    let completion = command(CommandName::LocalParticipationCompleted, None);

    // Act
    let changed = Change::NodeCollects.applied(&completion, &state());

    // Assert
    assert_eq!(changed.collecting_peers, NodeStates::peers(&["2", "1"]));
    assert_eq!(changed.pinging_peers, NodeStates::peers(&["3"]));
}

#[test]
fn applied_node_collects_with_its_own_readiness_already_accepted_changes_nothing() {
    // Arrange
    let completion = command(CommandName::LocalParticipationCompleted, None);
    let ready = NodeStates::of(Stage::Pinging, &[], &["1", "2"]);

    // Act
    let changed = Change::NodeCollects.applied(&completion, &ready);

    // Assert
    assert_eq!(changed, ready);
}

#[test]
fn applied_node_removed_before_its_completion_records_its_pinging_never_completed() {
    // Arrange
    let own_removal = command(CommandName::RemovalCommitted, Some("1"));
    let pinging = NodeStates::of(Stage::Pinging, &["1", "3"], &["2"]);

    // Act
    let changed = Change::NodeRemoved.applied(&own_removal, &pinging);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            members: NodeStates::peers(&["2", "3"]),
            pinging_peers: NodeStates::peers(&["3"]),
            ..pinging
        }
    );
}

#[test]
fn applied_node_removed_leaves_the_node_out_with_no_liveness_and_no_change_in_flight() {
    // Arrange
    let own_removal = command(CommandName::RemovalCommitted, Some("1"));
    let reconfiguring = NodeState {
        suspected: NodeStates::peers(&["3"]),
        misses: NodeStates::misses(&[("2", 1)]),
        last_probed: Some(String::from("2")),
        ..NodeStates::reconfiguring(Reconfiguration::RemovalRequested(String::from("1")))
    };

    // Act
    let changed = Change::NodeRemoved.applied(&own_removal, &reconfiguring);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            members: NodeStates::peers(&["2", "3"]),
            collecting_peers: Vec::new(),
            completed_before_removal: true,
            ..NodeStates::reconfiguring(Reconfiguration::None)
        }
    );
}

#[test]
fn applied_nothing_leaves_the_state_as_it_was() {
    // Arrange
    let request = command(CommandName::JoinRequested, Some("8"));

    // Act
    let changed = Change::Nothing.applied(&request, &state());

    // Assert
    assert_eq!(changed, state());
}

#[test]
fn applied_peer_collects_makes_the_peer_the_last_collecting_peer() {
    // Arrange
    let readiness = command(CommandName::ReadyObserved, Some("3"));

    // Act
    let changed = Change::PeerCollects.applied(&readiness, &state());

    // Assert
    assert_eq!(changed.collecting_peers, NodeStates::peers(&["2", "3"]));
    assert_eq!(changed.pinging_peers, NodeStates::peers(&["3"]));
}

#[test]
fn applied_peer_joins_makes_the_peer_the_last_member() {
    // Arrange
    let approval = command(CommandName::JoinApproved, Some("8"));

    // Act
    let changed = Change::PeerJoins.applied(&approval, &state());

    // Assert
    assert_eq!(changed.members, NodeStates::peers(&["1", "2", "3", "8"]));
    assert_eq!(changed.collecting_peers, NodeStates::peers(&["2"]));
}

#[test]
fn applied_peer_pings_makes_the_peer_the_last_pinging_peer() {
    // Arrange
    let participation = command(CommandName::ParticipationObserved, Some("2"));

    // Act
    let changed = Change::PeerPings.applied(&participation, &state());

    // Assert
    assert_eq!(changed.pinging_peers, NodeStates::peers(&["3", "2"]));
    assert_eq!(changed.collecting_peers, NodeStates::peers(&["2"]));
}

#[test]
fn applied_peer_removed_forgets_a_suspect_and_keeps_the_rest_of_the_liveness() {
    // Arrange
    let commit = command(CommandName::RemovalCommitted, Some("2"));
    let concluded = NodeState {
        suspected: NodeStates::peers(&["2"]),
        misses: NodeStates::misses(&[("3", 1)]),
        last_probed: Some(String::from("3")),
        ..NodeStates::concluded()
    };

    // Act
    let changed = Change::PeerRemoved.applied(&commit, &concluded);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            members: NodeStates::peers(&["1", "3"]),
            suspected: Vec::new(),
            ..concluded
        }
    );
}

#[test]
fn applied_peer_removed_of_the_member_probed_last_has_probed_no_one() {
    // Arrange
    let commit = command(CommandName::RemovalCommitted, Some("2"));
    let concluded = NodeState {
        misses: NodeStates::misses(&[("2", 1)]),
        last_probed: Some(String::from("2")),
        ..NodeStates::concluded()
    };

    // Act
    let changed = Change::PeerRemoved.applied(&commit, &concluded);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            members: NodeStates::peers(&["1", "3"]),
            ..NodeStates::concluded()
        }
    );
}

#[test]
fn applied_peer_removed_takes_the_peer_out_of_the_members_and_the_peers_gathered() {
    // Arrange
    let commit = command(CommandName::RemovalCommitted, Some("2"));
    let pinging = NodeStates::of(Stage::Pinging, &["2", "3"], &["2"]);

    // Act
    let changed = Change::PeerRemoved.applied(&commit, &pinging);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            members: NodeStates::peers(&["1", "3"]),
            pinging_peers: NodeStates::peers(&["3"]),
            collecting_peers: Vec::new(),
            ..pinging
        }
    );
}

#[test]
fn applied_peer_revived_no_longer_suspects_the_peer() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["2", "3"]),
        ..NodeStates::concluded()
    };

    // Act
    let changed = Change::PeerRevived.applied(&command(CommandName::PeerHeard, Some("2")), &state);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            suspected: NodeStates::peers(&["3"]),
            ..NodeStates::concluded()
        }
    );
}

#[test]
fn applied_peer_suspected_makes_the_peer_the_last_suspected_and_forgets_its_misses() {
    // Arrange
    let state = NodeState {
        suspected: NodeStates::peers(&["3"]),
        misses: NodeStates::misses(&[("2", 2)]),
        ..NodeStates::concluded()
    };

    // Act
    let changed = Change::PeerSuspected.applied(
        &command(CommandName::IndirectProbesMissed, Some("2")),
        &state,
    );

    // Assert
    assert_eq!(
        changed,
        NodeState {
            suspected: NodeStates::peers(&["3", "2"]),
            ..NodeStates::concluded()
        }
    );
}

#[test]
fn applied_probe_moves_makes_the_member_after_the_last_probed_the_one_probed_last() {
    // Arrange
    let state = NodeStates::concluded();

    // Act
    let changed = Change::ProbeMoves.applied(&command(CommandName::ProbeDue, None), &state);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            last_probed: Some(String::from("2")),
            ..NodeStates::concluded()
        }
    );
}

#[test]
fn applied_removal_completes_takes_the_peer_out_and_ends_the_removal() {
    // Arrange
    let commit = command(CommandName::RemovalCommitted, Some("2"));
    let requested = NodeStates::reconfiguring(Reconfiguration::RemovalRequested(String::from("2")));

    // Act
    let changed = Change::RemovalCompletes.applied(&commit, &requested);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            members: NodeStates::peers(&["1", "3"]),
            ..NodeStates::reconfiguring(Reconfiguration::None)
        }
    );
}

#[test]
fn applied_removal_pends_requests_the_removal_of_the_peer() {
    // Arrange
    let request = command(CommandName::RemovalRequested, Some("2"));
    let concluded = NodeStates::concluded();

    // Act
    let changed = Change::RemovalPends.applied(&request, &concluded);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            reconfiguration: Reconfiguration::RemovalRequested(String::from("2")),
            ..concluded
        }
    );
}

#[test]
fn applied_removal_withdrawn_leaves_no_change_in_flight() {
    // Arrange
    let abort = command(CommandName::RemovalAborted, Some("2"));
    let requested = NodeStates::reconfiguring(Reconfiguration::RemovalRequested(String::from("2")));

    // Act
    let changed = Change::RemovalWithdrawn.applied(&abort, &requested);

    // Assert
    assert_eq!(changed, NodeStates::reconfiguring(Reconfiguration::None));
}
