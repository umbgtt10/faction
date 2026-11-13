// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::timer::Timer;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol_validation::in_memory_timer::InMemoryTimer;

#[test]
fn cancel_scheduled_event_removes_only_that_event() {
    // Arrange
    let mut timer = InMemoryTimer::new();
    timer.schedule(TimerEvent::Fire(TimerMessage::RetryPing));
    timer.schedule(TimerEvent::Fire(TimerMessage::RetryReady));

    // Act
    timer.cancel(TimerEvent::Fire(TimerMessage::RetryPing));

    // Assert
    assert_eq!(
        timer.poll(),
        Some(TimerEvent::Fire(TimerMessage::RetryReady))
    );
    assert_eq!(timer.poll(), None);
}

#[test]
fn new_holds_no_event() {
    // Arrange
    let mut timer = InMemoryTimer::new();

    // Act
    let event = timer.poll();

    // Assert
    assert_eq!(event, None);
}

#[test]
fn poll_returns_events_in_scheduling_order() {
    // Arrange
    let mut timer = InMemoryTimer::new();
    timer.schedule(TimerEvent::Fire(TimerMessage::DeadlineExpired));
    timer.schedule(TimerEvent::Fire(TimerMessage::RetryPing));

    // Act
    let first = timer.poll();
    let second = timer.poll();

    // Assert
    assert_eq!(first, Some(TimerEvent::Fire(TimerMessage::DeadlineExpired)));
    assert_eq!(second, Some(TimerEvent::Fire(TimerMessage::RetryPing)));
}
