// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::reconfiguration::Reconfiguration;
use faction_chaos::harness::committed_log::CommittedLog;
use faction_chaos::model::change::Change;
use faction_chaos::oracle::expected_end::ExpectedEnd;
use faction_system_tests::membership::Membership;
use faction_system_tests::node_snapshot::NodeSnapshot;

// The end of a log that adds 3 and removes 1 from three, two required.
fn expected() -> ExpectedEnd {
    let mut log = CommittedLog::new(Membership {
        peers: vec![0, 1, 2],
        required_count: 2,
        epoch: 0,
    });
    log.commit(Change::Addition { peer_id: 3 });
    log.commit(Change::Removal { peer_id: 1 });
    ExpectedEnd::new(&log)
}

fn member() -> NodeSnapshot {
    NodeSnapshot {
        peer_state: PeerState::Bootstrapped,
        member_count: 3,
        members: vec![0, 2, 3],
        required_count: 2,
        reconfiguration: Reconfiguration::None,
        suspected: Vec::new(),
        quorum: Quorum::Held,
        epoch: 2,
    }
}

fn removed(epoch: u64) -> NodeSnapshot {
    NodeSnapshot {
        peer_state: PeerState::Removed,
        epoch,
        ..member()
    }
}

fn waiting() -> NodeSnapshot {
    NodeSnapshot {
        peer_state: PeerState::Fresh,
        member_count: 0,
        members: Vec::new(),
        required_count: 0,
        reconfiguration: Reconfiguration::None,
        suspected: Vec::new(),
        quorum: Quorum::NotTracked,
        epoch: 0,
    }
}

#[test]
fn departure_of_a_member_at_the_end_state_is_none() {
    // Arrange
    let expected = expected();

    // Act
    let departure = expected.departure(0, &member());

    // Assert
    assert_eq!(departure, None);
}

#[test]
fn departure_of_a_member_behind_names_the_end_state() {
    // Arrange
    let expected = expected();
    let snapshot = NodeSnapshot {
        epoch: 1,
        ..member()
    };

    // Act
    let departure = expected.departure(2, &snapshot);

    // Assert
    assert_eq!(departure, Some(format!("{:?}", member())));
}

#[test]
fn departure_of_a_member_holding_its_members_in_another_order_is_none() {
    // Arrange
    let expected = expected();
    let snapshot = NodeSnapshot {
        members: vec![3, 0, 2],
        ..member()
    };

    // Act
    let departure = expected.departure(3, &snapshot);

    // Assert
    assert_eq!(departure, None);
}

#[test]
fn departure_of_a_member_still_suspecting_names_the_end_state() {
    // Arrange
    let expected = expected();
    let snapshot = NodeSnapshot {
        suspected: vec![3],
        ..member()
    };

    // Act
    let departure = expected.departure(2, &snapshot);

    // Assert
    assert!(departure.is_some());
}

#[test]
fn departure_of_a_newcomer_the_log_never_adds_and_running_names_its_waiting() {
    // Arrange
    let expected = expected();

    // Act
    let departure = expected.departure(4, &member());

    // Assert
    assert_eq!(departure, Some(format!("{:?}", waiting())));
}

#[test]
fn departure_of_a_newcomer_the_log_never_adds_still_waiting_is_none() {
    // Arrange
    let expected = expected();

    // Act
    let departure = expected.departure(4, &waiting());

    // Assert
    assert_eq!(departure, None);
}

#[test]
fn departure_of_a_node_removed_and_still_running_names_its_removal() {
    // Arrange
    let expected = expected();

    // Act
    let departure = expected.departure(1, &member());

    // Assert
    assert_eq!(departure, Some(String::from("Removed at epoch 2")));
}

#[test]
fn departure_of_a_node_removed_at_its_removal_is_none() {
    // Arrange
    let expected = expected();

    // Act
    let departure = expected.departure(1, &removed(2));

    // Assert
    assert_eq!(departure, None);
}
