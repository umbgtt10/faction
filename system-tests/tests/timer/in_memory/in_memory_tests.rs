// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::timer_trait::Timer;
use faction_system_tests::timer::in_memory::in_memory_timer::InMemoryTimer;

fn some_event() -> TimerEvent {
    TimerEvent::Fire(TimerMessage::RetryPing)
}

fn other_event() -> TimerEvent {
    TimerEvent::Fire(TimerMessage::RetryReady)
}

#[test]
fn schedule_then_poll_returns_event() {
    // Arrange
    let mut timer = InMemoryTimer::new();
    let event = some_event();

    // Act
    timer.schedule(event.clone());

    // Assert
    assert_eq!(timer.poll(), Some(event));
}

#[test]
fn poll_empty_returns_none() {
    // Arrange
    let mut timer = InMemoryTimer::new();

    // Act & Assert
    assert_eq!(timer.poll(), None);
}

#[test]
fn fifo_order() {
    // Arrange
    let mut timer = InMemoryTimer::new();

    // Act
    timer.schedule(some_event());
    timer.schedule(other_event());

    // Assert
    assert_eq!(timer.poll(), Some(some_event()));
    assert_eq!(timer.poll(), Some(other_event()));
    assert_eq!(timer.poll(), None);
}

#[test]
fn cancel_removes_matching_event() {
    // Arrange
    let mut timer = InMemoryTimer::new();
    timer.schedule(some_event());
    timer.schedule(other_event());

    // Act
    timer.cancel(some_event());

    // Assert
    assert_eq!(timer.poll(), Some(other_event()));
    assert_eq!(timer.poll(), None);
}

#[test]
fn cancel_non_existent_is_noop() {
    // Arrange
    let mut timer = InMemoryTimer::new();
    timer.schedule(some_event());

    // Act
    timer.cancel(other_event());

    // Assert
    assert_eq!(timer.poll(), Some(some_event()));
}

#[test]
fn cancel_twice_is_idempotent() {
    // Arrange
    let mut timer = InMemoryTimer::new();
    timer.schedule(some_event());

    // Act
    timer.cancel(some_event());
    timer.cancel(some_event());

    // Assert
    assert!(timer.poll().is_none());
}
