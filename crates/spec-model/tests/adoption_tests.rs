// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::adoption::Adoption;
use faction_spec_model::command::Command;
use faction_spec_model::node_state::NodeState;
use faction_spec_model::reconfiguration::Reconfiguration;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::stage::Stage;

#[test]
fn adopted_of_a_membership_keeps_every_list_to_its_members_and_ends_the_change() {
    // Arrange
    let tracked = NodeState {
        pinging_peers: NodeStates::peers(&["3", "2"]),
        suspected: NodeStates::peers(&["3"]),
        misses: NodeStates::misses(&[("2", 1), ("3", 2)]),
        last_probed: Some(String::from("3")),
        reconfigurations: vec![Reconfiguration::RemovalRequested(String::from("3"), None)],
        ..NodeStates::of(Stage::Reconfiguring, &[], &["1", "3"])
    };
    let membership = Command {
        name: CommandName::MembershipReceived,
        peer_id: None,
        required_count: Some(1),
        leaves: None,
        epoch: 6,
        members: NodeStates::peers(&["2", "1"]),
    };

    // Act
    let adopted = Adoption::adopted(&membership, &tracked);

    // Assert
    assert_eq!(
        adopted,
        NodeState {
            members: NodeStates::peers(&["2", "1"]),
            pinging_peers: NodeStates::peers(&["2"]),
            collecting_peers: NodeStates::peers(&["1"]),
            suspected: Vec::new(),
            misses: NodeStates::misses(&[("2", 1)]),
            last_probed: None,
            reconfigurations: Vec::new(),
            required_count: 1,
            epoch: 6,
            ..tracked
        }
    );
}
