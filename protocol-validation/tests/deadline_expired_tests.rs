// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol::timer_message::TimerMessage;

use faction_protocol_validation::cluster::Cluster;
use rstest::rstest;

#[rstest]
#[case(3, 2)]
#[case(4, 3)]
#[case(5, 3)]
#[case(5, 4)]
#[case(6, 4)]
#[case(7, 4)]
#[case(7, 5)]
#[case(8, 5)]
#[case(9, 5)]
#[case(9, 6)]
#[case(10, 6)]
#[case(10, 7)]
#[case(11, 6)]
#[case(12, 7)]
#[case(13, 9)]
#[case(3, 3)]
#[case(4, 4)]
#[case(5, 5)]
#[case(6, 6)]
#[case(8, 8)]
#[case(2, 2)]
fn all_nodes_time_out_when_deadline_fires_before_quorum(
    #[case] size: usize,
    #[case] quorum: usize,
) {
    // Arrange
    let mut cluster = Cluster::new(size, quorum);
    cluster.start_all();
    for i in 0..size {
        cluster.inject_timer(i, TimerMessage::DeadlineExpired);
    }

    // Act — each node fires its own timers until the injected deadline arrives
    let steps = 2 * size + 5;
    for i in 0..size {
        for _ in 0..steps {
            cluster.step_timer_node(i);
        }
    }

    // Assert — the missed deadline is recorded (reported as TimedOut), but the
    // node stays receptive: its retry timers keep running so it can still converge.
    for i in 0..size {
        assert!(
            cluster.is_timed_out(i),
            "node {i} (size {size}, quorum {quorum}) did not report TimedOut"
        );
    }
}
