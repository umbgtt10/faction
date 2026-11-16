// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::timer::Timer;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_system_tests::infra::live_policy::LivePolicy;
use faction_system_tests::infra::timer::faulty::faulty_timer::FaultyTimer;
use faction_system_tests::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;
use faction_system_tests::infra::timer::real::real_timer::RealTimer;
use faction_system_tests::infra::timer::simulated::simulated_clock::SimulatedClock;
use std::sync::Arc;
use std::time::Duration;

fn ping() -> TimerEvent {
    TimerEvent::Fire(TimerMessage::RetryPing)
}

fn ready() -> TimerEvent {
    TimerEvent::Fire(TimerMessage::RetryReady)
}

fn timer(
    policy: TimerFaultPolicy,
) -> (
    FaultyTimer,
    Arc<SimulatedClock>,
    LivePolicy<TimerFaultPolicy>,
) {
    let clock = Arc::new(SimulatedClock::at(Duration::ZERO));
    let live = LivePolicy::new(policy);
    let inner = RealTimer::with_delay(clock.clone(), Duration::from_millis(10));
    (
        FaultyTimer::new(0, live.clone(), Box::new(inner)),
        clock,
        live,
    )
}

#[test]
fn cancel_kept_withdraws_a_held_firing() {
    // Arrange
    let (mut timer, clock, _) = timer(TimerFaultPolicy {
        late: 100,
        ..TimerFaultPolicy::default()
    });
    timer.schedule(ping());
    clock.advance(Duration::from_millis(10));
    let held = timer.poll();

    // Act
    timer.cancel(ping());
    let after: Vec<Option<TimerEvent>> = (0..10).map(|_| timer.poll()).collect();

    // Assert
    assert_eq!(held, None);
    assert_eq!(after, vec![None; 10]);
}

#[test]
fn cancel_lost_lets_the_cancelled_firing_come() {
    // Arrange
    let (mut timer, clock, _) = timer(TimerFaultPolicy {
        lost_cancel: 100,
        ..TimerFaultPolicy::default()
    });
    timer.schedule(ping());

    // Act
    timer.cancel(ping());
    clock.advance(Duration::from_millis(10));

    // Assert
    assert_eq!(timer.poll(), Some(ping()));
}

#[test]
fn poll_of_a_dead_timer_side_loses_its_firings_for_good() {
    // Arrange
    let (mut timer, clock, live) = timer(TimerFaultPolicy {
        dead: true,
        ..TimerFaultPolicy::default()
    });
    timer.schedule(ping());
    clock.advance(Duration::from_millis(10));
    let dead = timer.poll();
    live.set(TimerFaultPolicy::default());

    // Act
    let healed = timer.poll();

    // Assert
    assert_eq!(dead, None);
    assert_eq!(healed, None);
}

#[test]
fn poll_of_a_duplicated_firing_delivers_it_again_two_polls_later() {
    // Arrange
    let (mut timer, clock, _) = timer(TimerFaultPolicy {
        duplicate: 100,
        ..TimerFaultPolicy::default()
    });
    timer.schedule(ping());
    clock.advance(Duration::from_millis(10));

    // Act
    let polled: Vec<Option<TimerEvent>> = (0..4).map(|_| timer.poll()).collect();

    // Assert
    assert_eq!(polled, vec![Some(ping()), None, Some(ping()), None]);
}

#[test]
fn poll_of_a_faithful_policy_passes_a_due_firing_through() {
    // Arrange
    let (mut timer, clock, _) = timer(TimerFaultPolicy::default());
    timer.schedule(ping());
    clock.advance(Duration::from_millis(10));

    // Act
    let polled = timer.poll();

    // Assert
    assert_eq!(polled, Some(ping()));
}

#[test]
fn poll_of_a_late_firing_holds_it_for_eight_polls() {
    // Arrange
    let (mut timer, clock, _) = timer(TimerFaultPolicy {
        late: 100,
        ..TimerFaultPolicy::default()
    });
    timer.schedule(ping());
    clock.advance(Duration::from_millis(10));

    // Act
    let polled: Vec<Option<TimerEvent>> = (0..9).map(|_| timer.poll()).collect();

    // Assert
    let mut expected = vec![None; 8];
    expected.push(Some(ping()));
    assert_eq!(polled, expected);
}

#[test]
fn poll_of_a_starved_timer_side_keeps_its_firings_for_later() {
    // Arrange
    let (mut timer, clock, live) = timer(TimerFaultPolicy {
        starved: true,
        ..TimerFaultPolicy::default()
    });
    timer.schedule(ping());
    clock.advance(Duration::from_millis(10));
    let starved = timer.poll();
    live.set(TimerFaultPolicy::default());

    // Act
    let fed = timer.poll();

    // Assert
    assert_eq!(starved, None);
    assert_eq!(fed, Some(ping()));
}

#[test]
fn poll_of_reordered_firings_delivers_the_later_one_first() {
    // Arrange
    let (mut timer, clock, _) = timer(TimerFaultPolicy {
        reorder: 100,
        seed: 7,
        ..TimerFaultPolicy::default()
    });
    timer.schedule(ping());
    timer.schedule(ready());
    clock.advance(Duration::from_millis(10));

    // Act
    let delivered: Vec<TimerEvent> = (0..8).filter_map(|_| timer.poll()).collect();

    // Assert
    assert_eq!(delivered, vec![ready(), ping()]);
}

#[test]
fn schedule_adding_keeps_the_pending_firing_beside_the_new() {
    // Arrange
    let (mut timer, clock, _) = timer(TimerFaultPolicy::default());
    timer.schedule(ping());
    clock.advance(Duration::from_millis(5));

    // Act
    timer.schedule(ping());
    clock.advance(Duration::from_millis(5));
    let first = timer.poll();
    clock.advance(Duration::from_millis(5));
    let second = timer.poll();

    // Assert
    assert_eq!(first, Some(ping()));
    assert_eq!(second, Some(ping()));
}

#[test]
fn schedule_replacing_postpones_the_pending_firing() {
    // Arrange
    let (mut timer, clock, _) = timer(TimerFaultPolicy {
        replace: true,
        ..TimerFaultPolicy::default()
    });
    timer.schedule(ping());
    clock.advance(Duration::from_millis(5));

    // Act
    timer.schedule(ping());
    clock.advance(Duration::from_millis(5));
    let first = timer.poll();
    clock.advance(Duration::from_millis(5));
    let second = timer.poll();
    let third = timer.poll();

    // Assert
    assert_eq!(first, None);
    assert_eq!(second, Some(ping()));
    assert_eq!(third, None);
}
