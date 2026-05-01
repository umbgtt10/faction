// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol::timer_message::TimerMessage;

use faction_protocol_validation::cluster::Cluster;

#[test]
fn deadline_expired_exits_with_timed_out_and_cancels_pending_timers() {
    // Arrange
    let mut cluster = Cluster::new(5, 4);
    cluster.start_all();
    cluster.inject_timer(0, TimerMessage::DeadlineExpired);

    // Act — 4× ParticipationObserved, LPC, RetryPing, then DeadlineExpired
    for _ in 0..7 {
        cluster.step_timer_node(0);
    }

    // Assert — node 0 is TimedOut with no pending timer events
    assert!(cluster.is_timed_out(0));
    assert!(!cluster.step_timer_node(0));
}

#[test]
fn all_nodes_time_out_when_deadline_fires_before_quorum() {
    // Arrange
    let mut cluster = Cluster::new(5, 4);
    cluster.start_all();
    for i in 0..5 {
        cluster.inject_timer(i, TimerMessage::DeadlineExpired);
    }

    // Act — each node processes 4× PO, LPC, RetryPing, then DeadlineExpired
    for i in 0..5 {
        for _ in 0..7 {
            cluster.step_timer_node(i);
        }
    }

    // Assert — all nodes are TimedOut with no pending timer events
    for i in 0..5 {
        assert!(cluster.is_timed_out(i));
        assert!(!cluster.step_timer_node(i));
    }
}
