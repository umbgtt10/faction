// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::fake_time;
use crate::support::real_time;
use crate::support::recipe;
use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::reconfiguration::Reconfiguration;
use faction_protocol::commit::Commit;
use faction_system_tests::cluster::Cluster;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::infra::fault_board::FaultBoard;
use faction_system_tests::infra::fault_settings::FaultSettings;
use faction_system_tests::infra::invariants::breaches::Breaches;
use faction_system_tests::infra::proposal::Proposal;
use faction_system_tests::infra::timer::faulty::clock_fault::ClockFault;
use faction_system_tests::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;
use faction_system_tests::infra::timer::simulated::simulated_clock::SimulatedClock;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;
use faction_system_tests::infra::transport::in_memory::in_memory_transport::InMemoryTransport;
use faction_system_tests::join::committer::Committer;
use faction_system_tests::node::Node;
use faction_system_tests::poll_expired::PollExpired;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::timer_delay::TimerDelay;
use faction_system_tests::traits::monotonic_clock::MonotonicClock;
use std::sync::Arc;
use std::time::Duration;

fn cluster() -> (Cluster, Arc<SimulatedClock>) {
    let (time, clock) = fake_time();
    let recipe = recipe(2, &time);
    let nodes: Vec<Node> = InMemoryTransport::new_mesh(&[0, 1])
        .into_iter()
        .zip([0, 1])
        .map(|(transport, id)| recipe.node(Spawn::Task, id, vec![0, 1], Box::new(transport)))
        .collect();
    let cluster = Cluster::new(
        nodes,
        vec![0, 1],
        Spawn::Task,
        TimerDelay::Minimal,
        None,
        None,
        time,
    );
    (cluster, clock)
}

#[test]
fn abort_addition_of_a_requested_addition_withdraws_it() {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 2, real_time()).build();
    cluster.poll_until_bootstrapped();
    cluster.add_newcomer(3);
    cluster.request_addition(1, 3);
    let requested = cluster.snapshot(1).reconfigurations;

    // Act
    cluster.abort_addition(1, 3);

    // Assert
    assert_eq!(
        requested,
        [Reconfiguration::AdditionRequested { peer_id: 3 }]
    );
    assert!(cluster.snapshot(1).reconfigurations.is_empty());
}

#[test]
fn abort_removal_of_a_requested_removal_withdraws_it() {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 2, real_time()).build();
    cluster.poll_until_bootstrapped();
    cluster.request_removal(1, 2);
    cluster.poll_until_removal_proposed(1, 2);

    // Act
    cluster.abort_removal(1, 2);

    // Assert
    assert!(cluster.snapshot(1).reconfigurations.is_empty());
    assert_eq!(cluster.snapshot(1).member_count, 3);
}

#[test]
fn commit_removal_reports_the_count_and_the_removal_to_every_node() {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 3, real_time()).build();
    cluster.poll_until_bootstrapped();

    // Act
    cluster.commit_removal(2, Some(2));

    // Assert
    for index in 0..2 {
        let snapshot = cluster.snapshot(index);
        assert_eq!((snapshot.member_count, snapshot.required_count), (2, 2));
        assert_eq!(snapshot.quorum, Quorum::Held);
    }
    assert_eq!(cluster.node_state(2), PeerState::Removed);
}

#[test]
fn cut_of_the_only_link_keeps_the_two_from_concluding() {
    // Arrange
    let (mut cluster, _) = cluster();
    cluster.cut(0, 1);

    // Act
    cluster.settle(50);

    // Assert
    assert!(!cluster.is_bootstrapped());
}

#[test]
fn new_holds_the_nodes_it_is_given() {
    // Arrange & Act
    let (cluster, _) = cluster();

    // Assert
    assert_eq!(cluster.node_count(), 2);
    assert_eq!(cluster.node_state(0), PeerState::Fresh);
}

#[test]
fn poll_within_a_cluster_healed_of_a_lossy_link_concludes() {
    // Arrange
    let (time, _) = fake_time();
    let board = FaultBoard::default();
    let mut cluster = ClusterBuilder::new(3, 3, time)
        .faults(board.clone())
        .build();
    board.node(0).transport.set(FaultPolicy {
        loss: 100,
        ..FaultPolicy::default()
    });
    cluster.settle(100);
    let stalled = cluster.is_bootstrapped();
    board.node(0).transport.set(FaultPolicy::default());

    // Act
    let polled = cluster.poll_within(
        Duration::from_secs(10),
        "every node bootstrapped",
        |cluster| cluster.is_bootstrapped(),
    );

    // Assert
    assert!(!stalled);
    assert_eq!(polled, Ok(()));
}

#[test]
fn poll_within_a_cluster_watched_for_breaches_through_a_change_finds_none() {
    // Arrange
    let (time, _) = fake_time();
    let breaches = Breaches::default();
    let mut cluster = ClusterBuilder::new(3, 2, time)
        .breaches(breaches.clone())
        .build();
    cluster.poll_until_bootstrapped();
    cluster.add(
        0,
        3,
        Committer::Commit {
            required_count: None,
        },
    );
    cluster.commit_removal(1, None);

    // Act
    let polled = cluster.poll_within(
        Duration::from_secs(10),
        "every member bootstrapped",
        |cluster| {
            [0, 2, 3]
                .into_iter()
                .all(|index| cluster.node_state(index) == PeerState::Bootstrapped)
        },
    );

    // Assert
    assert_eq!(polled, Ok(()));
    assert_eq!(cluster.node_state(1), PeerState::Removed);
    assert_eq!(breaches.first(), None);
}

#[test]
fn poll_within_a_cluster_whose_starved_timer_side_is_fed_concludes() {
    // Arrange
    let (time, _) = fake_time();
    let board = FaultBoard::default();
    let mut cluster = ClusterBuilder::new(3, 3, time)
        .faults(board.clone())
        .build();
    board.node(0).timer.set(TimerFaultPolicy {
        starved: true,
        ..TimerFaultPolicy::default()
    });
    cluster.settle(100);
    let stalled = cluster.is_bootstrapped();
    board.node(0).timer.set(TimerFaultPolicy::default());

    // Act
    let polled = cluster.poll_within(
        Duration::from_secs(10),
        "every node bootstrapped",
        |cluster| cluster.is_bootstrapped(),
    );

    // Assert
    assert!(!stalled);
    assert_eq!(polled, Ok(()));
}

#[test]
fn poll_within_a_cluster_whose_stopped_clock_runs_again_concludes() {
    // Arrange
    let (time, _) = fake_time();
    let board = FaultBoard::default();
    let mut cluster = ClusterBuilder::new(3, 3, time)
        .faults(board.clone())
        .build();
    board.node(0).clock.set(ClockFault {
        drift_ppm: -1_000_000,
        ..ClockFault::default()
    });
    cluster.settle(100);
    let stalled = cluster.is_bootstrapped();
    board.node(0).clock.set(ClockFault::default());

    // Act
    let polled = cluster.poll_within(
        Duration::from_secs(10),
        "every node bootstrapped",
        |cluster| cluster.is_bootstrapped(),
    );

    // Assert
    assert!(!stalled);
    assert_eq!(polled, Ok(()));
}

#[test]
fn poll_within_a_condition_met_returns_before_the_deadline() {
    // Arrange
    let (mut cluster, clock) = cluster();

    // Act
    let polled = cluster.poll_within(
        Duration::from_secs(10),
        "every node bootstrapped",
        |cluster| cluster.is_bootstrapped(),
    );

    // Assert
    assert_eq!(polled, Ok(()));
    assert!(cluster.is_bootstrapped());
    assert!(clock.now() < Duration::from_secs(10));
}

#[test]
fn poll_within_a_condition_never_met_gives_up_at_the_deadline_naming_it() {
    // Arrange
    let (mut cluster, clock) = cluster();

    // Act
    let polled = cluster.poll_within(Duration::from_millis(100), "the impossible", |_| false);

    // Assert
    assert_eq!(
        polled,
        Err(PollExpired {
            waiting_for: String::from("the impossible"),
            deadline: Duration::from_millis(100),
        })
    );
    assert_eq!(clock.now(), Duration::from_millis(100));
}

#[test]
fn report_commit_reports_the_commit_to_that_node_alone() {
    // Arrange
    let (cluster, _) = cluster();

    // Act
    cluster.report_commit(
        1,
        Commit::Addition {
            peer_id: 5,
            epoch: 1,
        },
    );

    // Assert
    assert_eq!(cluster.snapshot(1).members, vec![0, 1, 5]);
    assert_eq!(cluster.snapshot(1).epoch, 1);
    assert_eq!(cluster.snapshot(0).members, vec![0, 1]);
}

#[test]
fn report_decline_reports_the_decline_to_that_node_alone() {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 2, real_time()).build();
    cluster.poll_until_bootstrapped();
    cluster.request_removal(1, 2);
    cluster.poll_until_removal_proposed(1, 2);
    let proposed = cluster.take_proposals(1);

    // Act
    cluster.report_decline(1, 2, 0);

    // Assert
    let removal = Proposal::Removal {
        peer_id: 2,
        epoch: 0,
    };
    assert_eq!(proposed, vec![removal]);
    assert_eq!(cluster.take_proposals(1), vec![removal]);
    assert!(cluster.take_proposals(0).is_empty());
}

#[test]
fn set_faults_of_a_node_holds_the_cluster_back_until_healed() {
    // Arrange
    let (time, _) = fake_time();
    let mut cluster = ClusterBuilder::new(3, 3, time).build();
    cluster.set_faults(
        0,
        &FaultSettings {
            transport: FaultPolicy {
                mute: true,
                ..FaultPolicy::default()
            },
            ..FaultSettings::default()
        },
    );
    cluster.settle(100);
    let stalled = cluster.is_bootstrapped();

    // Act
    cluster.set_faults(0, &FaultSettings::default());

    // Assert
    let polled = cluster.poll_within(
        Duration::from_secs(10),
        "every node bootstrapped",
        |cluster| cluster.is_bootstrapped(),
    );
    assert!(!stalled);
    assert_eq!(polled, Ok(()));
}

#[test]
fn set_faults_of_a_thread_node_holds_the_cluster_back_until_healed() {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 3, real_time())
        .spawn(Spawn::Thread)
        .build();
    cluster.set_faults(
        1,
        &FaultSettings {
            transport: FaultPolicy {
                mute: true,
                ..FaultPolicy::default()
            },
            ..FaultSettings::default()
        },
    );
    cluster.settle(20);
    let stalled = cluster.is_bootstrapped();

    // Act
    cluster.set_faults(1, &FaultSettings::default());

    // Assert
    cluster.poll_until_bootstrapped();
    assert!(!stalled);
}

#[test]
fn settle_of_a_thread_cluster_past_the_lifetime_it_was_given_leaves_it_unconcluded() {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 3, real_time())
        .spawn(Spawn::Thread)
        .lifetime(Duration::from_millis(1))
        .build();

    // Act
    cluster.settle(10);

    // Assert
    assert!(!cluster.is_bootstrapped());
}

#[test]
fn settle_pauses_once_a_round() {
    // Arrange
    let (mut cluster, clock) = cluster();

    // Act
    cluster.settle(3);

    // Assert
    assert_eq!(clock.now(), Duration::from_millis(30));
}

#[test]
fn silence_of_a_node_keeps_the_cluster_from_concluding() {
    // Arrange
    let (mut cluster, _) = cluster();
    cluster.silence(0);

    // Act
    cluster.settle(50);

    // Assert
    assert!(!cluster.is_bootstrapped());
}

#[test]
fn silent_builds_a_cluster_that_runs_its_faults_unannounced() {
    // Arrange
    let (time, _) = fake_time();
    let board = FaultBoard::default();
    board.node(0).transport.set(FaultPolicy {
        duplication: 100,
        ..FaultPolicy::default()
    });

    // Act
    let mut cluster = ClusterBuilder::new(3, 3, time)
        .faults(board)
        .silent()
        .build();

    // Assert
    let polled = cluster.poll_within(
        Duration::from_secs(10),
        "every node bootstrapped",
        |cluster| cluster.is_bootstrapped(),
    );
    assert_eq!(polled, Ok(()));
}

#[test]
fn step_all_steps_every_node() {
    // Arrange
    let (mut cluster, clock) = cluster();
    cluster.start_all();
    clock.advance(Duration::from_millis(10));

    // Act
    cluster.step_all();

    // Assert
    assert_eq!(cluster.node_state(0), PeerState::Pinging);
    assert_eq!(cluster.node_state(1), PeerState::Pinging);
}

#[test]
fn step_of_a_node_with_a_due_timer_applies_it() {
    // Arrange
    let (mut cluster, clock) = cluster();
    cluster.start_all();
    clock.advance(Duration::from_millis(10));

    // Act
    let stepped = cluster.step(0);

    // Assert
    assert!(stepped);
    assert_eq!(cluster.node_state(0), PeerState::Pinging);
    assert_eq!(cluster.node_state(1), PeerState::Fresh);
}

#[test]
fn step_of_a_node_with_nothing_due_does_nothing() {
    // Arrange
    let (mut cluster, _) = cluster();
    cluster.start_all();
    while cluster.step(0) {}

    // Act
    let stepped = cluster.step(0);

    // Assert
    assert!(!stepped);
}

#[test]
fn unsilence_lets_a_silenced_cluster_conclude() {
    // Arrange
    let (mut cluster, _) = cluster();
    cluster.silence(0);
    cluster.settle(50);

    // Act
    cluster.unsilence(0);
    cluster.poll_until(Cluster::is_bootstrapped);

    // Assert
    assert!(cluster.is_bootstrapped());
}
