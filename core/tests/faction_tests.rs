// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;

#[test]
fn get_snapshot_returns_snapshot_available_with_initial_state() {
    // Arrange
    let config = Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4));
    let observer = Box::new(NoOpObserver);
    let mut faction = Faction::new(config, observer);

    // Act
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => panic!("expected Probed"),
    };

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Fresh);
    assert_eq!(cluster_view.exit_mode(), None);
    assert!(!cluster_view.is_pinging_completed());
    assert!(!cluster_view.is_exited());
    assert_eq!(cluster_view.pinging_peers().len(), 0);
    assert_eq!(cluster_view.collecting_peers().len(), 0);
}

#[test]
fn get_snapshot_does_not_mutate_state() {
    // Arrange
    let config = Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4));
    let observer = Box::new(NoOpObserver);
    let mut faction = Faction::new(config, observer);

    // Act
    let first = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    let _ = faction.process(Command::Probe);
    let second = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(first, second);
}

#[test]
fn get_snapshot_works_after_valid_inputs() {
    // Arrange
    let config = Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4));
    let observer = Box::new(NoOpObserver);
    let mut faction = Faction::new(config, observer);
    let _ = faction.process(Command::ParticipationObserved { peer_id: 0 });

    // Act
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => panic!("expected Probed"),
    };

    // Assert
    assert_eq!(cluster_view.pinging_peers().len(), 1);
}
