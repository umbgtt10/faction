// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::command::Command;
use faction_spec_model::node_state::NodeState;
use faction_spec_model::rules::change::Change;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::stage::Stage;

fn command(name: CommandName, peer_id: Option<&str>) -> Command {
    Command {
        name,
        peer_id: peer_id.map(String::from),
    }
}

fn state() -> NodeState {
    NodeStates::of(Stage::Pinging, &["3"], &["2"])
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
