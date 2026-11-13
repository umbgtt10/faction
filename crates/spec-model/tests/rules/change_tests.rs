// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::addition::Addition;
use faction_spec_model::command::Command;
use faction_spec_model::node_state::NodeState;
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
    let requested = NodeStates::reconfiguring(Addition::Requested(String::from("4")));

    // Act
    let changed = Change::AdditionAcknowledged.applied(&acknowledgement, &requested);

    // Assert
    assert_eq!(
        changed,
        NodeStates::reconfiguring(Addition::Acknowledged(String::from("4")))
    );
}

#[test]
fn applied_addition_completes_makes_the_peer_the_last_member_and_ends_the_addition() {
    // Arrange
    let commit = command(CommandName::AdditionCommitted, Some("4"));
    let acknowledged = NodeStates::reconfiguring(Addition::Acknowledged(String::from("4")));

    // Act
    let changed = Change::AdditionCompletes.applied(&commit, &acknowledged);

    // Assert
    assert_eq!(
        changed,
        NodeState {
            members: NodeStates::peers(&["1", "2", "3", "4"]),
            ..NodeStates::reconfiguring(Addition::None)
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
            addition: Addition::Requested(String::from("4")),
            ..concluded
        }
    );
}

#[test]
fn applied_addition_withdrawn_leaves_no_addition_in_flight() {
    // Arrange
    let abort = command(CommandName::AdditionAborted, Some("4"));
    let acknowledged = NodeStates::reconfiguring(Addition::Acknowledged(String::from("4")));

    // Act
    let changed = Change::AdditionWithdrawn.applied(&abort, &acknowledged);

    // Assert
    assert_eq!(changed, NodeStates::reconfiguring(Addition::None));
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
