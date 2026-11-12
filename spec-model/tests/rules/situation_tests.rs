// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::node_states::NodeStates;
use faction_spec_model::command::Command;
use faction_spec_model::rules::situation::Situation;
use faction_spec_model::vocabulary::command_name::CommandName;
use faction_spec_model::vocabulary::stage::Stage;

const ADMITTED: [CommandName; 2] = [CommandName::ReadyObserved, CommandName::Probe];

fn completion() -> Command {
    Command {
        name: CommandName::LocalParticipationCompleted,
        peer_id: None,
    }
}

fn readiness_from(peer_id: &str) -> Command {
    Command {
        name: CommandName::ReadyObserved,
        peer_id: Some(String::from(peer_id)),
    }
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
