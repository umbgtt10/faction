// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;

#[test]
fn process_probe_returns_cluster_view_with_initial_state() {
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
    assert_eq!(cluster_view.conclusion(), None);
    assert!(!cluster_view.is_pinging_completed());
    assert!(!cluster_view.is_concluded());
    assert_eq!(cluster_view.pinging_peers().len(), 0);
    assert_eq!(cluster_view.collecting_peers().len(), 0);
}

#[test]
fn process_probe_does_not_mutate_state() {
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
fn process_probe_works_after_valid_inputs() {
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

fn all_commands() -> Vec<Command> {
    vec![
        Command::ParticipationObserved { peer_id: 0 },
        Command::ReadyObserved { peer_id: 0 },
        Command::LocalParticipationCompleted,
        Command::DeadlineExpired,
        Command::Probe,
    ]
}

#[test]
fn process_accepted_into_non_terminal_reports_full_admissible_set() {
    // Arrange
    let config = Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(5));
    let mut faction = Faction::new(config, Box::new(NoOpObserver));

    // Act
    let admissible = match faction.process(Command::ParticipationObserved { peer_id: 1 }) {
        ProcessResult::Accepted { admissible, .. } => admissible,
        _ => panic!("expected Accepted"),
    };

    // Assert
    assert_eq!(admissible, all_commands());
}

#[test]
fn process_accepted_into_bootstrapped_reports_probe_only() {
    // Arrange
    let config = Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(5));
    let mut faction = Faction::new(config, Box::new(NoOpObserver));
    let _ = faction.process(Command::ParticipationObserved { peer_id: 99 });
    let _ = faction.process(Command::LocalParticipationCompleted);
    let _ = faction.process(Command::ReadyObserved { peer_id: 1 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 2 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 3 });

    // Act
    let admissible = match faction.process(Command::ReadyObserved { peer_id: 4 }) {
        ProcessResult::Accepted { admissible, .. } => admissible,
        _ => panic!("expected Accepted at quorum"),
    };

    // Assert
    assert_eq!(admissible, vec![Command::Probe]);
}

#[test]
fn process_accepted_admissible_equals_subsequent_probe() {
    // Arrange
    let config = Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(5));
    let mut faction = Faction::new(config, Box::new(NoOpObserver));

    // Act
    let accepted_admissible = match faction.process(Command::ParticipationObserved { peer_id: 1 }) {
        ProcessResult::Accepted { admissible, .. } => admissible,
        _ => panic!("expected Accepted"),
    };
    let probed_admissible = match faction.process(Command::Probe) {
        ProcessResult::Probed { admissible, .. } => admissible,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(accepted_admissible, probed_admissible);
}
