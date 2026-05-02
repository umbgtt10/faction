// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::time::Duration;

use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::timer_trait::Timer;
use faction_system_tests::timer::real::real_timer::RealTimer;

fn some_event() -> TimerEvent {
    TimerEvent::Fire(TimerMessage::RetryPing)
}

fn other_event() -> TimerEvent {
    TimerEvent::Fire(TimerMessage::RetryReady)
}

#[test]
fn poll_empty_returns_none() {
    // Arrange
    let mut timer = RealTimer::with_delay(Duration::from_millis(10));

    // Act & Assert
    assert_eq!(timer.poll(), None);
}

#[test]
fn schedule_does_not_return_immediately() {
    // Arrange
    let mut timer = RealTimer::with_delay(Duration::from_millis(100));

    // Act & Assert
    timer.schedule(some_event());
    assert_eq!(timer.poll(), None);
}

#[test]
fn schedule_returns_event_after_delay() {
    // Arrange
    let mut timer = RealTimer::with_delay(Duration::from_millis(10));

    // Act
    timer.schedule(some_event());
    std::thread::sleep(Duration::from_millis(20));

    // Assert
    assert_eq!(timer.poll(), Some(some_event()));
}

#[test]
fn fifo_order_when_deadlines_match() {
    // Arrange
    let mut timer = RealTimer::with_delay(Duration::from_millis(1));

    // Act
    timer.schedule(some_event());
    timer.schedule(other_event());
    std::thread::sleep(Duration::from_millis(10));

    // Assert
    assert_eq!(timer.poll(), Some(some_event()));
    assert_eq!(timer.poll(), Some(other_event()));
    assert_eq!(timer.poll(), None);
}

#[test]
fn cancel_removes_matching_event() {
    // Arrange
    let mut timer = RealTimer::with_delay(Duration::from_millis(10));
    timer.schedule(some_event());
    timer.schedule(other_event());

    // Act
    timer.cancel(some_event());
    std::thread::sleep(Duration::from_millis(20));

    // Assert
    assert_eq!(timer.poll(), Some(other_event()));
    assert_eq!(timer.poll(), None);
}

#[test]
fn cancel_non_existent_is_noop() {
    // Arrange
    let mut timer = RealTimer::with_delay(Duration::from_millis(10));
    timer.schedule(some_event());

    // Act
    timer.cancel(other_event());
    std::thread::sleep(Duration::from_millis(20));

    // Assert
    assert_eq!(timer.poll(), Some(some_event()));
}

#[test]
fn only_returns_events_whose_deadline_has_passed() {
    // Arrange
    let mut timer = RealTimer::with_delay(Duration::from_millis(50));
    timer.schedule(some_event());

    // Act & Assert — not ready yet
    assert_eq!(timer.poll(), None);

    // Act & Assert — after delay
    std::thread::sleep(Duration::from_millis(60));
    assert_eq!(timer.poll(), Some(some_event()));
}

#[test]
fn multiple_delays_respected() {
    // Arrange
    let mut timer = RealTimer::with_delay(Duration::from_millis(20));
    timer.schedule(some_event());
    std::thread::sleep(Duration::from_millis(30));

    // Act
    timer.schedule(other_event());

    // Assert — first event ready immediately, second needs another 20ms
    assert_eq!(timer.poll(), Some(some_event()));
    assert_eq!(timer.poll(), None);

    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(timer.poll(), Some(other_event()));
}

#[test]
fn default_delay_is_50ms() {
    // Arrange
    let mut timer = RealTimer::new();

    // Act
    timer.schedule(some_event());
    std::thread::sleep(Duration::from_millis(30));

    // Assert — 30ms < 50ms default, not ready yet
    assert_eq!(timer.poll(), None);

    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(timer.poll(), Some(some_event()));
}
