// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol::timer_message::TimerMessage;

use faction_system_tests::cluster::Cluster;

#[test]
fn deadline_expired_exits_with_timed_out_and_cancels_pending_timers() {
    // Arrange
    let mut cluster = Cluster::new(2, 2);
    cluster.start_all();
    cluster.inject_timer(0, TimerMessage::DeadlineExpired);

    // Act — ParticipationObserved, then LocalParticipationCompleted, then DeadlineExpired
    cluster.step_timer_node(0);
    cluster.step_timer_node(0);
    cluster.step_timer_node(0);

    // Assert — node 0 is TimedOut with no pending timer events
    assert!(cluster.is_timed_out(0));
    assert!(!cluster.step_timer_node(0));
}

#[test]
fn all_nodes_time_out_when_deadline_fires_before_quorum() {
    // Arrange
    let mut cluster = Cluster::new(3, 3);
    cluster.start_all();
    cluster.inject_timer(0, TimerMessage::DeadlineExpired);
    cluster.inject_timer(1, TimerMessage::DeadlineExpired);
    cluster.inject_timer(2, TimerMessage::DeadlineExpired);

    // Act — each node processes ParticipationObserved×2, LPC, then DeadlineExpired
    cluster.step_timer_node(0);
    cluster.step_timer_node(0);
    cluster.step_timer_node(0);
    cluster.step_timer_node(0);

    cluster.step_timer_node(1);
    cluster.step_timer_node(1);
    cluster.step_timer_node(1);
    cluster.step_timer_node(1);

    cluster.step_timer_node(2);
    cluster.step_timer_node(2);
    cluster.step_timer_node(2);
    cluster.step_timer_node(2);

    // Assert — all nodes are TimedOut with no pending timer events
    assert!(cluster.is_timed_out(0));
    assert!(cluster.is_timed_out(1));
    assert!(cluster.is_timed_out(2));
    assert!(!cluster.step_timer_node(0));
    assert!(!cluster.step_timer_node(1));
    assert!(!cluster.step_timer_node(2));
}
