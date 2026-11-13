// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::traits::monotonic_clock_tests::FakeMonotonicClock;
use faction_protocol::timer::Timer;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_system_tests::infra::timer::real::real_timer::RealTimer;
use std::sync::Arc;
use std::time::Duration;

fn clock() -> Arc<FakeMonotonicClock> {
    Arc::new(FakeMonotonicClock::at(Duration::ZERO))
}

fn other_event() -> TimerEvent {
    TimerEvent::Fire(TimerMessage::RetryReady)
}

fn some_event() -> TimerEvent {
    TimerEvent::Fire(TimerMessage::RetryPing)
}

#[test]
fn cancel_non_existent_is_noop() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delay(clock.clone(), Duration::from_millis(10));
    timer.schedule(some_event());

    // Act
    timer.cancel(other_event());
    clock.advance(Duration::from_millis(20));

    // Assert
    assert_eq!(timer.poll(), Some(some_event()));
}

#[test]
fn cancel_removes_matching_event() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delay(clock.clone(), Duration::from_millis(10));
    timer.schedule(some_event());
    timer.schedule(other_event());

    // Act
    timer.cancel(some_event());
    clock.advance(Duration::from_millis(20));

    // Assert
    assert_eq!(timer.poll(), Some(other_event()));
    assert_eq!(timer.poll(), None);
}

#[test]
fn fifo_order_when_deadlines_match() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delay(clock.clone(), Duration::from_millis(1));

    // Act
    timer.schedule(some_event());
    timer.schedule(other_event());
    clock.advance(Duration::from_millis(10));

    // Assert
    assert_eq!(timer.poll(), Some(some_event()));
    assert_eq!(timer.poll(), Some(other_event()));
    assert_eq!(timer.poll(), None);
}

#[test]
fn new_default_delay_is_50ms() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::new(clock.clone());

    // Act
    timer.schedule(some_event());
    clock.advance(Duration::from_millis(30));

    // Assert — 30ms < 50ms default, not ready yet
    assert_eq!(timer.poll(), None);

    clock.advance(Duration::from_millis(30));
    assert_eq!(timer.poll(), Some(some_event()));
}

#[test]
fn poll_empty_returns_none() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delay(clock.clone(), Duration::from_millis(10));

    // Act & Assert
    assert_eq!(timer.poll(), None);
}

#[test]
fn schedule_does_not_return_immediately() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delay(clock.clone(), Duration::from_millis(100));

    // Act & Assert
    timer.schedule(some_event());
    assert_eq!(timer.poll(), None);
}

#[test]
fn schedule_multiple_delays_respected() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delay(clock.clone(), Duration::from_millis(20));
    timer.schedule(some_event());
    clock.advance(Duration::from_millis(30));

    // Act
    timer.schedule(other_event());

    // Assert — first event ready immediately, second needs another 20ms
    assert_eq!(timer.poll(), Some(some_event()));
    assert_eq!(timer.poll(), None);

    clock.advance(Duration::from_millis(30));
    assert_eq!(timer.poll(), Some(other_event()));
}

#[test]
fn schedule_only_returns_when_deadline_passed() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delay(clock.clone(), Duration::from_millis(50));
    timer.schedule(some_event());

    // Act & Assert — not ready yet
    assert_eq!(timer.poll(), None);

    // Act & Assert — after delay
    clock.advance(Duration::from_millis(60));
    assert_eq!(timer.poll(), Some(some_event()));
}

#[test]
fn schedule_returns_event_after_delay() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delay(clock.clone(), Duration::from_millis(10));

    // Act
    timer.schedule(some_event());
    clock.advance(Duration::from_millis(20));

    // Assert
    assert_eq!(timer.poll(), Some(some_event()));
}

#[test]
fn with_delays_deadline_waits_its_own_delay() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delays(
        clock.clone(),
        Duration::from_millis(10),
        Duration::from_millis(100),
    );
    timer.schedule(TimerEvent::Fire(TimerMessage::DeadlineExpired));
    timer.schedule(some_event());

    // Act
    clock.advance(Duration::from_millis(10));

    // Assert
    assert_eq!(timer.poll(), Some(some_event()));
    assert_eq!(timer.poll(), None);

    // Act
    clock.advance(Duration::from_millis(90));

    // Assert
    assert_eq!(
        timer.poll(),
        Some(TimerEvent::Fire(TimerMessage::DeadlineExpired))
    );
}

#[test]
fn with_delays_indirect_probes_missed_waits_eight_delays() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delays(
        clock.clone(),
        Duration::from_millis(10),
        Duration::from_millis(1000),
    );
    timer.schedule(TimerEvent::Fire(TimerMessage::IndirectProbesMissed {
        peer_id: 1,
    }));
    clock.advance(Duration::from_millis(79));
    let early = timer.poll();

    // Act
    clock.advance(Duration::from_millis(1));

    // Assert
    assert_eq!(early, None);
    assert_eq!(
        timer.poll(),
        Some(TimerEvent::Fire(TimerMessage::IndirectProbesMissed {
            peer_id: 1
        }))
    );
}

#[test]
fn with_delays_probe_due_waits_five_delays() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delays(
        clock.clone(),
        Duration::from_millis(10),
        Duration::from_millis(1000),
    );
    timer.schedule(TimerEvent::Fire(TimerMessage::ProbeDue));
    clock.advance(Duration::from_millis(49));
    let early = timer.poll();

    // Act
    clock.advance(Duration::from_millis(1));

    // Assert
    assert_eq!(early, None);
    assert_eq!(timer.poll(), Some(TimerEvent::Fire(TimerMessage::ProbeDue)));
}

#[test]
fn with_delays_probe_missed_waits_four_delays() {
    // Arrange
    let clock = clock();
    let mut timer = RealTimer::with_delays(
        clock.clone(),
        Duration::from_millis(10),
        Duration::from_millis(1000),
    );
    timer.schedule(TimerEvent::Fire(TimerMessage::ProbeMissed { peer_id: 1 }));
    clock.advance(Duration::from_millis(39));
    let early = timer.poll();

    // Act
    clock.advance(Duration::from_millis(1));

    // Assert
    assert_eq!(early, None);
    assert_eq!(
        timer.poll(),
        Some(TimerEvent::Fire(TimerMessage::ProbeMissed { peer_id: 1 }))
    );
}
