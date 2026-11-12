// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::recording::Recording;
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
    let observations: Observations = Arc::new(Mutex::new(Recording::default()));
    let faction = Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4)),
        Box::new(RecordingObserver::new(Arc::clone(&observations))),
    );
    (faction, observations)
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
fn process_observes_accepted_command_only_as_a_transition() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let input = Command::ParticipationObserved { peer_id: 1 };

    // Act
    let _ = faction.process(input);

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.transitions.len(), 1);
    assert_eq!(obs.transitions[0].0, input);
    assert!(obs.queries.is_empty());
    assert!(obs.rejections.is_empty());
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
    assert_eq!(obs.transitions.len(), 2);
    let (observed_input, transition) = &obs.transitions[1];
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, transition.outputs);
    assert_eq!(transition.previous_view.peer_state(), PeerState::Pinging);
    assert!(!transition.previous_view.is_pinging_completed());
    assert!(!transition.previous_view.is_concluded());
    assert_eq!(
        transition.outputs,
        &[Outcome::DeadlineMissed { confirmed_count: 0 }]
    );
    assert_eq!(transition.new_view.peer_state(), PeerState::TimedOut);
    assert_eq!(transition.new_view.conclusion(), None);
    assert!(!transition.new_view.is_concluded());
    assert!(!transition.new_view.is_pinging_completed());
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
    assert_eq!(obs.transitions.len(), 3);
    let (observed_input, transition) = &obs.transitions[2];
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, transition.outputs);
    assert_eq!(transition.previous_view.peer_state(), PeerState::Collecting);
    assert!(!transition.previous_view.is_concluded());
    assert_eq!(transition.new_view.peer_state(), PeerState::TimedOut);
    assert_eq!(transition.new_view.conclusion(), None);
    assert!(!transition.new_view.is_concluded());
    assert!(transition.new_view.deadline_missed());
    assert_eq!(
        transition.outputs,
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
    assert_eq!(obs.transitions.len(), 2);
    let (observed_input, transition) = &obs.transitions[1];
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, transition.outputs);
    assert_eq!(
        transition.outputs,
        &[Outcome::DuplicateParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_view, transition.new_view);
    assert_eq!(transition.new_view.pinging_peers().len(), 1);
    assert_eq!(transition.new_view.peer_state(), PeerState::Pinging);
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
    assert_eq!(obs.transitions.len(), 4);
    let (observed_input, transition) = &obs.transitions[3];
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, transition.outputs);
    assert_eq!(
        transition.outputs,
        &[Outcome::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_view, transition.new_view);
    assert_eq!(transition.new_view.peer_state(), PeerState::Collecting);
    assert_eq!(transition.new_view.collecting_peers().len(), 2);
    assert!(!transition.new_view.is_concluded());
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
    assert_eq!(obs.transitions.len(), 2);
    let (observed_input, transition) = &obs.transitions[1];
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, transition.outputs);
    assert_eq!(
        transition.outputs,
        &[Outcome::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_view, transition.new_view);
    assert_eq!(transition.new_view.collecting_peers().len(), 1);
    assert_eq!(transition.new_view.peer_state(), PeerState::Pinging);
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
    assert_eq!(obs.transitions.len(), 2);
    let (observed_input, transition) = &obs.transitions[1];
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, transition.outputs);
    assert_eq!(transition.previous_view.peer_state(), PeerState::Pinging);
    assert!(!transition.previous_view.is_pinging_completed());
    assert!(!transition.previous_view.is_concluded());
    assert_eq!(transition.new_view.peer_state(), PeerState::Collecting);
    assert!(transition.new_view.is_pinging_completed());
    assert!(!transition.new_view.is_concluded());
    assert_eq!(transition.new_view.collecting_peers().len(), 1);
    assert_eq!(
        transition.outputs,
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
    let input = Command::ParticipationObserved { peer_id: 99 };

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.transitions.len(), 1);
    let (observed_input, transition) = &obs.transitions[0];
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, transition.outputs);
    assert_eq!(
        transition.outputs,
        &[Outcome::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(transition.previous_view.peer_state(), PeerState::Fresh);
    assert_eq!(transition.new_view.peer_state(), PeerState::Pinging);
    assert_eq!(transition.new_view.pinging_peers().len(), 0);
    assert!(!transition.new_view.is_concluded());
}

#[test]
fn process_observes_non_member_ready_from_initial() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let input = Command::ReadyObserved { peer_id: 99 };

    // Act
    let outcomes = match faction.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.transitions.len(), 1);
    let (observed_input, transition) = &obs.transitions[0];
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, transition.outputs);
    assert_eq!(
        transition.outputs,
        &[Outcome::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(transition.previous_view.peer_state(), PeerState::Fresh);
    assert_eq!(transition.new_view.peer_state(), PeerState::Pinging);
    assert_eq!(transition.new_view.collecting_peers().len(), 0);
    assert!(!transition.new_view.is_concluded());
}

#[test]
fn process_observes_probe_only_as_a_query() {
    // Arrange
    let (mut faction, observations) = recording_faction();
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });

    // Act
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        ProcessResult::Accepted { .. } | ProcessResult::Rejected { .. } => {
            panic!("expected probed")
        }
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(obs.queries, vec![(Command::Probe, cluster_view)]);
    assert_eq!(obs.transitions.len(), 1);
    assert!(obs.rejections.is_empty());
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
    assert_eq!(obs.transitions.len(), 4);
    let (observed_input, transition) = &obs.transitions[3];
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, transition.outputs);
    assert_eq!(
        transition.outputs,
        &[
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    assert_eq!(transition.previous_view.peer_state(), PeerState::Pinging);
    assert!(!transition.previous_view.is_pinging_completed());
    assert!(!transition.previous_view.is_concluded());
    assert_eq!(transition.new_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(
        transition.new_view.conclusion(),
        Some(Conclusion::Bootstrapped)
    );
    assert!(transition.new_view.is_concluded());
    assert_eq!(transition.new_view.collecting_peers().len(), 4);
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
    assert_eq!(obs.transitions.len(), 5);
    let (observed_input, transition) = &obs.transitions[4];
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, transition.outputs);
    assert_eq!(transition.previous_view.peer_state(), PeerState::Collecting);
    assert_eq!(transition.previous_view.collecting_peers().len(), 3);
    assert!(!transition.previous_view.is_concluded());
    assert_eq!(transition.new_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(
        transition.new_view.conclusion(),
        Some(Conclusion::Bootstrapped)
    );
    assert!(transition.new_view.is_concluded());
    assert_eq!(transition.new_view.collecting_peers().len(), 4);
    assert_eq!(
        transition.outputs,
        &[
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped
            },
        ]
    );
}

#[test]
fn process_observes_rejected_command_only_as_a_rejection() {
    // Arrange
    let (mut faction, observations) = recording_faction();

    // Act
    let (cluster_view, admissible) = match faction.process(Command::DeadlineExpired) {
        ProcessResult::Rejected {
            cluster_view,
            admissible,
        } => (cluster_view, admissible),
        ProcessResult::Accepted { .. } | ProcessResult::Probed { .. } => {
            panic!("expected rejected")
        }
    };

    // Assert
    let obs = observations.lock().unwrap();
    assert_eq!(
        obs.rejections,
        vec![(Command::DeadlineExpired, cluster_view, admissible)]
    );
    assert!(obs.transitions.is_empty());
    assert!(obs.queries.is_empty());
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
    assert_eq!(obs.transitions.len(), 4);
    let (observed_input, transition) = &obs.transitions[3];
    assert_eq!(*observed_input, input);
    assert_eq!(outcomes, transition.outputs);
    assert_eq!(transition.outputs, &[Outcome::ReadyAccepted { peer_id: 2 }]);
    assert_eq!(transition.previous_view.peer_state(), PeerState::Collecting);
    assert_eq!(transition.previous_view.collecting_peers().len(), 2);
    assert!(!transition.previous_view.is_concluded());
    assert_eq!(transition.new_view.peer_state(), PeerState::Collecting);
    assert_eq!(transition.new_view.collecting_peers().len(), 3);
    assert!(!transition.new_view.is_concluded());
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
    assert_eq!(obs.transitions.len(), 5);

    let (_, transition0) = &obs.transitions[0];
    assert_eq!(
        transition0.outputs,
        &[Outcome::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(outcomes_0, transition0.outputs);

    let (_, transition1) = &obs.transitions[1];
    assert_eq!(
        transition1.outputs,
        &[
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
    assert_eq!(outcomes_1, transition1.outputs);

    let (_, transition2) = &obs.transitions[2];
    assert_eq!(
        transition2.outputs,
        &[Outcome::ReadyAccepted { peer_id: 1 }]
    );
    assert_eq!(outcomes_2, transition2.outputs);

    let (_, transition3) = &obs.transitions[3];
    assert_eq!(
        transition3.outputs,
        &[Outcome::ReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(outcomes_3, transition3.outputs);

    let (_, transition4) = &obs.transitions[4];
    assert_eq!(
        transition4.outputs,
        &[
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    assert_eq!(outcomes_4, transition4.outputs);
    assert!(transition4.new_view.is_concluded());
}
