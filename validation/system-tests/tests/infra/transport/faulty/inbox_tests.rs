// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;
use faction_system_tests::infra::transport::faulty::inbox::Inbox;

fn lost_of(message: TransportMessage, policy: &FaultPolicy) -> usize {
    let mut inbox = Inbox::new(7);
    inbox.arrive(ping(0), policy);
    (0..100)
        .filter(|_| inbox.arrive(message.clone(), policy).is_some())
        .count()
}

fn membership(members: u64) -> TransportMessage {
    TransportMessage::Membership {
        from: 0,
        to: 2,
        members: (0..members).collect(),
        required_count: 1,
        epoch: 1,
    }
}

fn ping(epoch: u64) -> TransportMessage {
    TransportMessage::Ping { from: 1, epoch }
}

#[test]
fn arrive_at_a_backlog_loses_a_larger_message_more_often() {
    // Arrange
    let policy = FaultPolicy {
        size_loss: 30,
        ..FaultPolicy::default()
    };

    // Act
    let lost = [lost_of(ping(1), &policy), lost_of(membership(5), &policy)];

    // Assert
    assert!(lost[0] < lost[1], "{lost:?}");
}

#[test]
fn arrive_at_a_backlog_loses_a_message_by_its_size() {
    // Arrange
    let mut inbox = Inbox::new(7);
    let policy = FaultPolicy {
        size_loss: 100,
        ..FaultPolicy::default()
    };
    inbox.arrive(ping(1), &policy);

    // Act
    let lost = inbox.arrive(membership(5), &policy);

    // Assert
    assert_eq!(
        lost,
        Some(("SIZE (a fragment lost at a backlog)", membership(5)))
    );
}

#[test]
fn arrive_at_a_backlog_loses_a_message_by_the_backlog() {
    // Arrange
    let mut inbox = Inbox::new(7);
    let policy = FaultPolicy {
        burst_loss: 100,
        ..FaultPolicy::default()
    };
    inbox.arrive(ping(1), &policy);

    // Act
    let lost = inbox.arrive(ping(2), &policy);

    // Assert
    assert_eq!(lost, Some(("BURST (lost to a backlog)", ping(2))));
}

#[test]
fn arrive_at_a_backlog_of_two_loses_at_twice_the_percentage() {
    // Arrange
    let mut inbox = Inbox::new(7);
    inbox.arrive(ping(1), &FaultPolicy::default());
    inbox.arrive(ping(2), &FaultPolicy::default());
    let policy = FaultPolicy {
        burst_loss: 50,
        ..FaultPolicy::default()
    };

    // Act
    let lost = inbox.arrive(ping(3), &policy);

    // Assert
    assert_eq!(lost, Some(("BURST (lost to a backlog)", ping(3))));
}

#[test]
fn arrive_at_an_idle_ring_keeps_a_message_whatever_its_size() {
    // Arrange
    let mut inbox = Inbox::new(7);
    let policy = FaultPolicy {
        size_loss: 100,
        burst_loss: 100,
        ..FaultPolicy::default()
    };

    // Act
    let lost = inbox.arrive(membership(5), &policy);

    // Assert
    assert_eq!(lost, None);
    assert_eq!(inbox.take(), Some(membership(5)));
}

#[test]
fn arrive_past_its_capacity_drops_the_tail() {
    // Arrange
    let mut inbox = Inbox::new(7);
    let policy = FaultPolicy {
        inbox: 2,
        ..FaultPolicy::default()
    };
    inbox.arrive(ping(1), &policy);
    inbox.arrive(ping(2), &policy);

    // Act
    let lost = inbox.arrive(ping(3), &policy);

    // Assert
    assert_eq!(lost, Some(("INBOX FULL (tail dropped)", ping(3))));
}

#[test]
fn take_returns_the_messages_in_the_order_they_arrived() {
    // Arrange
    let mut inbox = Inbox::new(7);
    let policy = FaultPolicy::default();
    inbox.arrive(ping(1), &policy);
    inbox.arrive(ping(2), &policy);

    // Act
    let taken = [inbox.take(), inbox.take(), inbox.take()];

    // Assert
    assert_eq!(taken, [Some(ping(1)), Some(ping(2)), None]);
}
