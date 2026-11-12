// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::recording_observer::Observations;
use crate::support::recording_observer::RecordingObserver;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use std::sync::Arc;
use std::sync::Mutex;

fn all_commands() -> Vec<Command> {
    vec![
        Command::ParticipationObserved { peer_id: 0 },
        Command::ReadyObserved { peer_id: 0 },
        Command::LocalParticipationCompleted,
        Command::DeadlineExpired,
        Command::JoinRequested { peer_id: 0 },
        Command::JoinApproved { peer_id: 0 },
        Command::JoinRejected { peer_id: 0 },
        Command::Probe,
    ]
}

extern crate alloc;

fn fresh() -> Faction {
    Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4)),
        Box::new(NoOpObserver),
    )
}

fn probe(faction: &mut Faction) -> ClusterView {
    match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    }
}

fn process(faction: &mut Faction, command: Command) -> Vec<Outcome> {
    match faction.process(command) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Rejected { .. } | ProcessResult::Probed { .. } => Vec::new(),
    }
}

fn recording_faction() -> (Faction, Observations) {
    let observations: Observations = Arc::new(Mutex::new(Vec::new()));
    let faction = Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4)),
        Box::new(RecordingObserver::new(Arc::clone(&observations))),
    );
    (faction, observations)
}

fn reported_outcomes(result: &ProcessResult) -> Vec<Outcome> {
    match result {
        ProcessResult::Accepted { outcomes, .. } => outcomes.clone(),
        ProcessResult::Rejected { .. } | ProcessResult::Probed { .. } => {
            panic!("expected an accepted result")
        }
    }
}

fn reported_view(result: &ProcessResult) -> ClusterView {
    result.clone().into_cluster_view()
}

#[test]
fn a_repeated_join_request_is_forwarded_each_time() {
    // Arrange
    let mut faction = fresh();

    // Act
    let first = process(&mut faction, Command::JoinRequested { peer_id: 99 });
    let second = process(&mut faction, Command::JoinRequested { peer_id: 99 });

    // Assert
    assert_eq!(first, vec![Outcome::EmitJoinRequest { peer_id: 99 }]);
    assert_eq!(second, vec![Outcome::EmitJoinRequest { peer_id: 99 }]);
}

#[test]
fn admission_does_not_change_the_quorum_threshold() {
    // Arrange
    let mut faction = fresh();

    // Act
    let admitted = process(&mut faction, Command::JoinApproved { peer_id: 99 });
    let required = probe(&mut faction).required_count();

    // Assert
    assert_eq!(admitted, vec![Outcome::MemberAdmitted { peer_id: 99 }]);
    assert_eq!(required, 4);
}

#[test]
fn admission_does_not_retroactively_count_earlier_participation() {
    // Arrange
    let mut faction = fresh();
    let ignored = process(&mut faction, Command::ParticipationObserved { peer_id: 99 });
    let admitted = process(&mut faction, Command::JoinApproved { peer_id: 99 });

    // Act
    let after_admit = probe(&mut faction);
    let accepted = process(&mut faction, Command::ParticipationObserved { peer_id: 99 });

    // Assert
    assert_eq!(ignored, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(admitted, vec![Outcome::MemberAdmitted { peer_id: 99 }]);
    assert_eq!(after_admit.pinging_peers().len(), 0);
    assert_eq!(
        accepted,
        vec![Outcome::ParticipationAccepted { peer_id: 99 }]
    );
}

#[test]
fn approving_the_local_member_is_a_duplicate() {
    // Arrange & Act
    let mut faction = fresh();
    let outcomes = process(&mut faction, Command::JoinApproved { peer_id: 0 });

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateMemberIgnored { peer_id: 0 }]
    );
}

#[test]
fn config_returns_the_configuration_given_to_new() {
    // Arrange
    let faction = Faction::new(
        Config::new(2, vec![1, 2, 3], QuorumPolicy::new(2)),
        Box::new(NoOpObserver),
    );

    // Act
    let config = faction.config();

    // Assert
    assert_eq!(config.peer_id(), 2);
    assert_eq!(config.peers(), &[1, 2, 3]);
    assert_eq!(config.required_count(), 2);
}

#[test]
fn join_rejected_does_not_remove_an_existing_member() {
    // Arrange
    let mut faction = fresh();
    let _ = process(&mut faction, Command::JoinApproved { peer_id: 99 });
    let _ = process(&mut faction, Command::ParticipationObserved { peer_id: 99 });

    // Act
    let denied = process(&mut faction, Command::JoinRejected { peer_id: 99 });
    let still_member = process(&mut faction, Command::ParticipationObserved { peer_id: 99 });

    // Assert
    assert_eq!(denied, vec![Outcome::JoinDenied { peer_id: 99 }]);
    assert_eq!(
        still_member,
        vec![Outcome::DuplicateParticipationIgnored { peer_id: 99 }]
    );
}

#[test]
fn participation_from_an_admitted_peer_is_accepted() {
    // Arrange
    let mut faction = fresh();
    let admitted = process(&mut faction, Command::JoinApproved { peer_id: 99 });

    // Act
    let accepted = process(&mut faction, Command::ParticipationObserved { peer_id: 99 });

    // Assert
    assert_eq!(admitted, vec![Outcome::MemberAdmitted { peer_id: 99 }]);
    assert_eq!(
        accepted,
        vec![Outcome::ParticipationAccepted { peer_id: 99 }]
    );
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

#[test]
fn process_accepted_into_bootstrapped_reports_rejoin_and_probe() {
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
    assert_eq!(
        admissible,
        vec![
            Command::ParticipationObserved { peer_id: 0 },
            Command::JoinRequested { peer_id: 0 },
            Command::JoinApproved { peer_id: 0 },
            Command::JoinRejected { peer_id: 0 },
            Command::Probe
        ]
    );
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
fn process_observes_a_probe_once_with_its_result() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let accepted = faction.process(Command::ParticipationObserved { peer_id: 1 });

    // Act
    let result = faction.process(Command::Probe);

    // Assert
    assert_eq!(
        *observations.lock().unwrap(),
        vec![
            (Command::ParticipationObserved { peer_id: 1 }, accepted),
            (Command::Probe, result),
        ]
    );
}

#[test]
fn process_observes_a_rejected_command_once_with_its_result() {
    // Arrange
    let (mut faction, observations) = recording_faction();

    // Act
    let result = faction.process(Command::DeadlineExpired);

    // Assert
    assert!(matches!(result, ProcessResult::Rejected { .. }));
    assert_eq!(
        *observations.lock().unwrap(),
        vec![(Command::DeadlineExpired, result)]
    );
}

#[test]
fn process_observes_an_accepted_command_once_with_its_result() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let input = Command::ParticipationObserved { peer_id: 1 };

    // Act
    let result = faction.process(input);

    // Assert
    assert_eq!(*observations.lock().unwrap(), vec![(input, result)]);
}

#[test]
fn process_observes_deadline_missed_from_pinging() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let input = Command::DeadlineExpired;

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 2);
    let (observed_input, reported) = &obs[1];
    let view = reported_view(reported);
    let previous = reported_view(&obs[0].1);
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, reported_outcomes(reported));
    assert_eq!(previous.peer_state(), PeerState::Pinging);
    assert!(!previous.is_pinging_completed());
    assert!(!previous.is_concluded());
    assert_eq!(
        reported_outcomes(reported),
        &[Outcome::DeadlineMissed { confirmed_count: 0 }]
    );
    assert_eq!(view.peer_state(), PeerState::TimedOut);
    assert_eq!(view.conclusion(), None);
    assert!(!view.is_concluded());
    assert!(!view.is_pinging_completed());
}

#[test]
fn process_observes_deadline_missed_transition() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let _ = faction.process(Command::LocalParticipationCompleted);
    let input = Command::DeadlineExpired;

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 3);
    let (observed_input, reported) = &obs[2];
    let view = reported_view(reported);
    let previous = reported_view(&obs[1].1);
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, reported_outcomes(reported));
    assert_eq!(previous.peer_state(), PeerState::Collecting);
    assert!(!previous.is_concluded());
    assert_eq!(view.peer_state(), PeerState::TimedOut);
    assert_eq!(view.conclusion(), None);
    assert!(!view.is_concluded());
    assert!(view.deadline_missed());
    assert_eq!(
        reported_outcomes(reported),
        &[Outcome::DeadlineMissed { confirmed_count: 1 }]
    );
}

#[test]
fn process_observes_duplicate_participation_transition_without_state_change() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let input = Command::ParticipationObserved { peer_id: 1 };

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 2);
    let (observed_input, reported) = &obs[1];
    let view = reported_view(reported);
    let previous = reported_view(&obs[0].1);
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, reported_outcomes(reported));
    assert_eq!(
        reported_outcomes(reported),
        &[Outcome::DuplicateParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(previous, view);
    assert_eq!(view.pinging_peers().len(), 1);
    assert_eq!(view.peer_state(), PeerState::Pinging);
}

#[test]
fn process_observes_duplicate_ready_from_collecting() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let _ = faction.process(Command::LocalParticipationCompleted);
    let _ = faction.process(Command::ReadyObserved { peer_id: 1 });
    let input = Command::ReadyObserved { peer_id: 1 };

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 4);
    let (observed_input, reported) = &obs[3];
    let view = reported_view(reported);
    let previous = reported_view(&obs[2].1);
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, reported_outcomes(reported));
    assert_eq!(
        reported_outcomes(reported),
        &[Outcome::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(previous, view);
    assert_eq!(view.peer_state(), PeerState::Collecting);
    assert_eq!(view.collecting_peers().len(), 2);
    assert!(!view.is_concluded());
}

#[test]
fn process_observes_duplicate_ready_from_pinging() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::ReadyObserved { peer_id: 1 });
    let input = Command::ReadyObserved { peer_id: 1 };

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 2);
    let (observed_input, reported) = &obs[1];
    let view = reported_view(reported);
    let previous = reported_view(&obs[0].1);
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, reported_outcomes(reported));
    assert_eq!(
        reported_outcomes(reported),
        &[Outcome::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(previous, view);
    assert_eq!(view.collecting_peers().len(), 1);
    assert_eq!(view.peer_state(), PeerState::Pinging);
}

#[test]
fn process_observes_local_participation_completion_transition() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let input = Command::LocalParticipationCompleted;

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 2);
    let (observed_input, reported) = &obs[1];
    let view = reported_view(reported);
    let previous = reported_view(&obs[0].1);
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, reported_outcomes(reported));
    assert_eq!(previous.peer_state(), PeerState::Pinging);
    assert!(!previous.is_pinging_completed());
    assert!(!previous.is_concluded());
    assert_eq!(view.peer_state(), PeerState::Collecting);
    assert!(view.is_pinging_completed());
    assert!(!view.is_concluded());
    assert_eq!(view.collecting_peers().len(), 1);
    assert_eq!(
        reported_outcomes(reported),
        &[
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
}

#[test]
fn process_observes_non_member_participation_from_initial() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::Probe);
    let input = Command::ParticipationObserved { peer_id: 99 };

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 2);
    let (observed_input, reported) = &obs[1];
    let view = reported_view(reported);
    let previous = reported_view(&obs[0].1);
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, reported_outcomes(reported));
    assert_eq!(
        reported_outcomes(reported),
        &[Outcome::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(previous.peer_state(), PeerState::Fresh);
    assert_eq!(view.peer_state(), PeerState::Pinging);
    assert_eq!(view.pinging_peers().len(), 0);
    assert!(!view.is_concluded());
}

#[test]
fn process_observes_non_member_ready_from_initial() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::Probe);
    let input = Command::ReadyObserved { peer_id: 99 };

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 2);
    let (observed_input, reported) = &obs[1];
    let view = reported_view(reported);
    let previous = reported_view(&obs[0].1);
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, reported_outcomes(reported));
    assert_eq!(
        reported_outcomes(reported),
        &[Outcome::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(previous.peer_state(), PeerState::Fresh);
    assert_eq!(view.peer_state(), PeerState::Pinging);
    assert_eq!(view.collecting_peers().len(), 0);
    assert!(!view.is_concluded());
}

#[test]
fn process_observes_quorum_exit_from_pinging() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::ReadyObserved { peer_id: 1 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 2 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 3 });
    let input = Command::LocalParticipationCompleted;

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 4);
    let (observed_input, reported) = &obs[3];
    let view = reported_view(reported);
    let previous = reported_view(&obs[2].1);
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, reported_outcomes(reported));
    assert_eq!(
        reported_outcomes(reported),
        &[
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    assert_eq!(previous.peer_state(), PeerState::Pinging);
    assert!(!previous.is_pinging_completed());
    assert!(!previous.is_concluded());
    assert_eq!(view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(view.conclusion(), Some(Conclusion::Bootstrapped));
    assert!(view.is_concluded());
    assert_eq!(view.collecting_peers().len(), 4);
}

#[test]
fn process_observes_quorum_exit_transition() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let _ = faction.process(Command::LocalParticipationCompleted);
    let _ = faction.process(Command::ReadyObserved { peer_id: 1 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 2 });
    let input = Command::ReadyObserved { peer_id: 3 };

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 5);
    let (observed_input, reported) = &obs[4];
    let view = reported_view(reported);
    let previous = reported_view(&obs[3].1);
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, reported_outcomes(reported));
    assert_eq!(previous.peer_state(), PeerState::Collecting);
    assert_eq!(previous.collecting_peers().len(), 3);
    assert!(!previous.is_concluded());
    assert_eq!(view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(view.conclusion(), Some(Conclusion::Bootstrapped));
    assert!(view.is_concluded());
    assert_eq!(view.collecting_peers().len(), 4);
    assert_eq!(
        reported_outcomes(reported),
        &[
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped
            },
        ]
    );
}

#[test]
fn process_observes_timely_ready_from_collecting_no_quorum() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let _ = faction.process(Command::LocalParticipationCompleted);
    let _ = faction.process(Command::ReadyObserved { peer_id: 1 });
    let input = Command::ReadyObserved { peer_id: 2 };

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 4);
    let (observed_input, reported) = &obs[3];
    let view = reported_view(reported);
    let previous = reported_view(&obs[2].1);
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, reported_outcomes(reported));
    assert_eq!(
        reported_outcomes(reported),
        &[Outcome::ReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(previous.peer_state(), PeerState::Collecting);
    assert_eq!(previous.collecting_peers().len(), 2);
    assert!(!previous.is_concluded());
    assert_eq!(view.peer_state(), PeerState::Collecting);
    assert_eq!(view.collecting_peers().len(), 3);
    assert!(!view.is_concluded());
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

#[test]
fn process_state_transition_outputs_fully_observable() {
    // Arrange
    let (mut faction, observations) = recording_faction();

    // Act
    let outcomes_0 = match faction.process(Command::ParticipationObserved { peer_id: 1 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let outcomes_1 = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let outcomes_2 = match faction.process(Command::ReadyObserved { peer_id: 1 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let outcomes_3 = match faction.process(Command::ReadyObserved { peer_id: 2 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let outcomes_4 = match faction.process(Command::ReadyObserved { peer_id: 3 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.len(), 5);

    let (_, reported0) = &obs[0];
    assert_eq!(
        reported_outcomes(reported0),
        &[Outcome::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(outcomes_0, reported_outcomes(reported0));

    let (_, reported1) = &obs[1];
    assert_eq!(
        reported_outcomes(reported1),
        &[
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
    assert_eq!(outcomes_1, reported_outcomes(reported1));

    let (_, reported2) = &obs[2];
    assert_eq!(
        reported_outcomes(reported2),
        &[Outcome::ReadyAccepted { peer_id: 1 }]
    );
    assert_eq!(outcomes_2, reported_outcomes(reported2));

    let (_, reported3) = &obs[3];
    assert_eq!(
        reported_outcomes(reported3),
        &[Outcome::ReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(outcomes_3, reported_outcomes(reported3));

    let (_, reported4) = &obs[4];
    assert_eq!(
        reported_outcomes(reported4),
        &[
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    assert_eq!(outcomes_4, reported_outcomes(reported4));
    assert!(reported_view(reported4).is_concluded());
}
