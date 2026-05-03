// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use std::sync::Arc;
use std::sync::Mutex;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::config::Config;
use faction::faction::Faction;
use faction::observer::Observer;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::transition::Transition;

type Observations = Arc<Mutex<Vec<(Command, Transition)>>>;

struct RecordingObserver {
    observations: Observations,
}

impl Observer for RecordingObserver {
    fn observe(&mut self, command: Command, transition: Transition) {
        self.observations
            .lock()
            .unwrap()
            .push((command, transition));
    }

    fn observe_query(&mut self, _command: Command, _cluster_view: ClusterView) {}

    fn observe_rejection(
        &mut self,
        _command: Command,
        _cluster_view: ClusterView,
        _admissible: Vec<Command>,
    ) {
    }
}

fn recording_faction() -> (Faction, Observations) {
    let observations: Observations = Arc::new(Mutex::new(Vec::new()));
    let observer = RecordingObserver {
        observations: Arc::clone(&observations),
    };
    let faction = Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4)),
        Box::new(observer),
    );
    (faction, observations)
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
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(transition.previous_view().peer_state(), PeerState::Pinging);
    assert!(!transition.previous_view().is_pinging_completed());
    assert!(!transition.previous_view().is_exited());
    assert_eq!(transition.new_view().peer_state(), PeerState::Collecting);
    assert!(transition.new_view().is_pinging_completed());
    assert!(!transition.new_view().is_exited());
    assert_eq!(transition.new_view().collecting_peers().len(), 1);
    assert_eq!(
        transition.outputs(),
        &[
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
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
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::DuplicateParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_view(), transition.new_view());
    assert_eq!(transition.new_view().pinging_peers().len(), 1);
    assert_eq!(transition.new_view().peer_state(), PeerState::Pinging);
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
    let (observed_input, transition) = &obs[4];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.previous_view().peer_state(),
        PeerState::Collecting
    );
    assert_eq!(transition.previous_view().collecting_peers().len(), 3);
    assert!(!transition.previous_view().is_exited());
    assert_eq!(transition.new_view().peer_state(), PeerState::Bootstrapped);
    assert_eq!(
        transition.new_view().exit_mode(),
        Some(Conclusion::Bootstrapped)
    );
    assert!(transition.new_view().is_exited());
    assert_eq!(transition.new_view().collecting_peers().len(), 4);
    assert_eq!(
        transition.outputs(),
        &[
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped
            },
        ]
    );
}

#[test]
fn process_observes_deadline_exit_transition() {
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
    let (observed_input, transition) = &obs[2];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.previous_view().peer_state(),
        PeerState::Collecting
    );
    assert!(!transition.previous_view().is_exited());
    assert_eq!(transition.new_view().peer_state(), PeerState::TimedOut);
    assert_eq!(
        transition.new_view().exit_mode(),
        Some(Conclusion::TimedOut)
    );
    assert!(transition.new_view().is_exited());
    assert_eq!(
        transition.outputs(),
        &[Outcome::Concluded {
            mode: Conclusion::TimedOut
        }]
    );
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

    let (_, transition0) = &obs[0];
    assert_eq!(
        transition0.outputs(),
        &[Outcome::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(&outcomes_0, transition0.outputs());

    let (_, transition1) = &obs[1];
    assert_eq!(
        transition1.outputs(),
        &[
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
    assert_eq!(&outcomes_1, transition1.outputs());

    let (_, transition2) = &obs[2];
    assert_eq!(
        transition2.outputs(),
        &[Outcome::ReadyAccepted { peer_id: 1 }]
    );
    assert_eq!(&outcomes_2, transition2.outputs());

    let (_, transition3) = &obs[3];
    assert_eq!(
        transition3.outputs(),
        &[Outcome::ReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(&outcomes_3, transition3.outputs());

    let (_, transition4) = &obs[4];
    assert_eq!(
        transition4.outputs(),
        &[
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    assert_eq!(&outcomes_4, transition4.outputs());
    assert!(transition4.new_view().is_exited());
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
    assert_eq!(obs.len(), 1);
    let (observed_input, transition) = &obs[0];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(transition.previous_view().peer_state(), PeerState::Fresh);
    assert_eq!(transition.new_view().peer_state(), PeerState::Pinging);
    assert_eq!(transition.new_view().pinging_peers().len(), 0);
    assert!(!transition.new_view().is_exited());
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
    assert_eq!(obs.len(), 1);
    let (observed_input, transition) = &obs[0];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(transition.previous_view().peer_state(), PeerState::Fresh);
    assert_eq!(transition.new_view().peer_state(), PeerState::Pinging);
    assert_eq!(transition.new_view().collecting_peers().len(), 0);
    assert!(!transition.new_view().is_exited());
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
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_view(), transition.new_view());
    assert_eq!(transition.new_view().collecting_peers().len(), 1);
    assert_eq!(transition.new_view().peer_state(), PeerState::Pinging);
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
    let (observed_input, transition) = &obs[3];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    assert_eq!(transition.previous_view().peer_state(), PeerState::Pinging);
    assert!(!transition.previous_view().is_pinging_completed());
    assert!(!transition.previous_view().is_exited());
    assert_eq!(transition.new_view().peer_state(), PeerState::Bootstrapped);
    assert_eq!(
        transition.new_view().exit_mode(),
        Some(Conclusion::Bootstrapped)
    );
    assert!(transition.new_view().is_exited());
    assert_eq!(transition.new_view().collecting_peers().len(), 4);
}

#[test]
fn process_observes_deadline_exit_from_pinging() {
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
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(transition.previous_view().peer_state(), PeerState::Pinging);
    assert!(!transition.previous_view().is_pinging_completed());
    assert!(!transition.previous_view().is_exited());
    assert_eq!(
        transition.outputs(),
        &[Outcome::Concluded {
            mode: Conclusion::TimedOut
        }]
    );
    assert_eq!(transition.new_view().peer_state(), PeerState::TimedOut);
    assert_eq!(
        transition.new_view().exit_mode(),
        Some(Conclusion::TimedOut)
    );
    assert!(transition.new_view().is_exited());
    assert!(!transition.new_view().is_pinging_completed());
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
    let (observed_input, transition) = &obs[3];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::ReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(
        transition.previous_view().peer_state(),
        PeerState::Collecting
    );
    assert_eq!(transition.previous_view().collecting_peers().len(), 2);
    assert!(!transition.previous_view().is_exited());
    assert_eq!(transition.new_view().peer_state(), PeerState::Collecting);
    assert_eq!(transition.new_view().collecting_peers().len(), 3);
    assert!(!transition.new_view().is_exited());
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
    let (observed_input, transition) = &obs[3];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_view(), transition.new_view());
    assert_eq!(transition.new_view().peer_state(), PeerState::Collecting);
    assert_eq!(transition.new_view().collecting_peers().len(), 2);
    assert!(!transition.new_view().is_exited());
}
