// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::engine::fault_schedule::FaultSchedule;
use faction_chaos::model::fault::Fault;
use faction_chaos::model::fault_window::FaultWindow;
use faction_chaos::model::restart_from::RestartFrom;
use faction_chaos::model::restart_plan::RestartPlan;
use faction_system_tests::infra::fault_board::FaultBoard;
use faction_system_tests::infra::timer::faulty::clock_fault::ClockFault;
use faction_system_tests::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;

fn dead() -> Fault {
    Fault::Timer(TimerFaultPolicy {
        dead: true,
        ..TimerFaultPolicy::default()
    })
}

fn down(peer_id: u64, at: u64, down_for: u64) -> RestartPlan {
    RestartPlan {
        peer_id,
        at,
        down_for,
        from: RestartFrom::Genesis,
    }
}

fn lossy(loss: u8) -> FaultPolicy {
    FaultPolicy {
        loss,
        ..FaultPolicy::default()
    }
}

fn schedule(windows: Vec<FaultWindow>, restarts: Vec<RestartPlan>) -> (FaultSchedule, FaultBoard) {
    let board = FaultBoard::default();
    (
        FaultSchedule::new(windows, restarts, vec![0, 1, 2], board.clone()),
        board,
    )
}

fn window(peer_id: u64, from: u64, until: u64, fault: Fault) -> FaultWindow {
    FaultWindow {
        peer_id,
        from,
        until,
        fault,
    }
}

#[test]
fn apply_leaves_a_node_outside_every_window_faithful() {
    // Arrange
    let (schedule, board) = schedule(
        vec![window(1, 5, 15, Fault::Transport(lossy(40)))],
        Vec::new(),
    );

    // Act
    schedule.apply(15);

    // Assert
    assert_eq!(board.node(1).transport.current(), FaultPolicy::default());
    assert_eq!(board.node(0).transport.current(), FaultPolicy::default());
}

#[test]
fn apply_mutes_deafens_and_kills_the_timer_side_of_a_node_down() {
    // Arrange
    let (schedule, board) = schedule(Vec::new(), vec![down(2, 5, 10)]);

    // Act
    schedule.apply(8);

    // Assert
    let transport = board.node(2).transport.current();
    assert!(transport.mute && transport.deaf);
    assert!(board.node(2).timer.current().dead);
    assert!(!board.node(1).timer.current().dead);
}

#[test]
fn apply_sets_the_fault_of_each_layer_in_force() {
    // Arrange
    let fast = ClockFault {
        drift_ppm: 1_000_000,
        ..ClockFault::default()
    };
    let (schedule, board) = schedule(
        vec![
            window(1, 5, 15, Fault::Transport(lossy(40))),
            window(1, 0, 15, Fault::Clock(fast)),
            window(0, 0, 15, dead()),
        ],
        Vec::new(),
    );

    // Act
    schedule.apply(10);

    // Assert
    assert_eq!(board.node(1).transport.current(), lossy(40));
    assert_eq!(board.node(1).clock.current(), fast);
    assert!(board.node(0).timer.current().dead);
}

#[test]
fn apply_takes_the_window_last_opened() {
    // Arrange
    let (schedule, board) = schedule(
        vec![
            window(1, 10, 50, Fault::Transport(lossy(30))),
            window(1, 0, 50, Fault::Transport(lossy(10))),
        ],
        Vec::new(),
    );

    // Act
    schedule.apply(20);

    // Assert
    assert_eq!(board.node(1).transport.current(), lossy(30));
}

#[test]
fn apply_wedges_a_node_under_head_of_line_behind_a_peer_that_cannot_answer() {
    // Arrange
    let blocking = FaultPolicy {
        head_of_line: true,
        ..FaultPolicy::default()
    };
    let deaf = FaultPolicy {
        deaf: true,
        ..FaultPolicy::default()
    };
    let (schedule, board) = schedule(
        vec![
            window(0, 0, 50, Fault::Transport(blocking)),
            window(1, 0, 50, Fault::Transport(deaf)),
        ],
        Vec::new(),
    );

    // Act
    schedule.apply(8);

    // Assert
    assert_eq!(board.node(0).transport.current().unreachable, vec![1]);
}

#[test]
fn apply_wedges_a_node_under_head_of_line_behind_the_peers_down() {
    // Arrange
    let blocking = FaultPolicy {
        head_of_line: true,
        ..FaultPolicy::default()
    };
    let (schedule, board) = schedule(
        vec![window(0, 0, 50, Fault::Transport(blocking))],
        vec![down(2, 5, 10)],
    );

    // Act
    schedule.apply(8);

    // Assert
    assert_eq!(board.node(0).transport.current().unreachable, vec![2]);
}

#[test]
fn heal_makes_every_layer_of_every_node_faithful() {
    // Arrange
    let (schedule, board) = schedule(
        vec![window(1, 0, 50, Fault::Transport(lossy(40)))],
        vec![down(2, 0, 50)],
    );
    schedule.apply(10);

    // Act
    schedule.heal();

    // Assert
    for peer_id in 0..3 {
        assert_eq!(
            board.node(peer_id).transport.current(),
            FaultPolicy::default()
        );
        assert_eq!(
            board.node(peer_id).timer.current(),
            TimerFaultPolicy::default()
        );
        assert_eq!(board.node(peer_id).clock.current(), ClockFault::default());
    }
}

#[test]
fn healed_by_restart_names_a_node_whose_downtime_or_dead_timer_ends_at_the_heal() {
    // Arrange
    let (schedule, _) = schedule(vec![window(0, 50, 100, dead())], vec![down(2, 90, 10)]);

    // Act
    let restarted = schedule.healed_by_restart(100);

    // Assert
    assert_eq!(restarted, vec![0, 2]);
}

#[test]
fn healed_by_restart_names_the_nodes_down_or_with_their_timer_side_dead() {
    // Arrange
    let (schedule, _) = schedule(vec![window(1, 50, 150, dead())], vec![down(2, 90, 20)]);

    // Act
    let restarted = schedule.healed_by_restart(100);

    // Assert
    assert_eq!(restarted, vec![1, 2]);
}

#[test]
fn restarts_at_the_end_of_a_dead_timer_window_restart_its_node_from_what_it_holds() {
    // Arrange
    let (schedule, _) = schedule(vec![window(1, 5, 20, dead())], Vec::new());

    // Act
    let restarts = schedule.restarts_at(20);

    // Assert
    assert_eq!(
        restarts,
        vec![RestartPlan {
            peer_id: 1,
            at: 20,
            down_for: 0,
            from: RestartFrom::Snapshot { behind: 0 },
        }]
    );
}

#[test]
fn restarts_at_the_end_of_a_downtime_restart_its_node() {
    // Arrange
    let (schedule, _) = schedule(Vec::new(), vec![down(2, 5, 10)]);

    // Act
    let restarts = [schedule.restarts_at(14), schedule.restarts_at(15)];

    // Assert
    assert_eq!(restarts, [Vec::new(), vec![down(2, 5, 10)]]);
}
