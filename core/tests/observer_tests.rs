// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::node_state::NodeState;
use faction::observer::Observer;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::transition::Transition;

type Observations = Rc<RefCell<Vec<(Command, Transition)>>>;

struct RecordingObserver {
    observations: Observations,
}

impl Observer for RecordingObserver {
    fn observe(&mut self, input: Command, transition: Transition) {
        self.observations.borrow_mut().push((input, transition));
    }
}

fn recording_coordinator() -> (Faction, Observations) {
    let observations: Observations = Rc::new(RefCell::new(Vec::new()));
    let observer = RecordingObserver {
        observations: Rc::clone(&observations),
    };
    let coordinator = Faction::new(
        Config::new(
            0,
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(4),
            FreshnessPolicy::new(2),
        ),
        Box::new(observer),
    );
    (coordinator, observations)
}

#[test]
fn apply_observes_local_participation_completion_transition() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = Command::LocalParticipationCompleted;

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 2);
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(transition.previous_state().node_state(), NodeState::Pinging);
    assert!(!transition.previous_state().is_pinging_completed());
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(transition.new_state().node_state(), NodeState::Collecting);
    assert!(transition.new_state().is_pinging_completed());
    assert!(!transition.new_state().readiness_exited());
    assert_eq!(transition.new_state().collecting_confirmed_count(), 1);
    assert_eq!(
        transition.outputs(),
        &[
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
}

#[test]
fn apply_observes_duplicate_participation_transition_without_state_change() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 2);
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::DuplicateParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().pinging_confirmed_count(), 1);
    assert_eq!(transition.new_state().node_state(), NodeState::Pinging);
}

#[test]
fn apply_observes_stale_ready_transition_without_state_change() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.process(Command::LocalParticipationCompleted);
    let input = Command::ReadyObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    };

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 3);
    let (observed_input, transition) = &obs[2];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::StaleReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().node_state(), NodeState::Collecting);
    assert_eq!(transition.new_state().collecting_confirmed_count(), 1);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_quorum_exit_transition() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.process(Command::LocalParticipationCompleted);
    let _ = coordinator.process(Command::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let input = Command::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 5);
    let (observed_input, transition) = &obs[4];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.previous_state().node_state(),
        NodeState::Collecting
    );
    assert_eq!(transition.previous_state().collecting_confirmed_count(), 3);
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(transition.new_state().node_state(), NodeState::Bootstrapped);
    assert_eq!(
        transition.new_state().exit_mode(),
        Some(ReadinessExitMode::Bootstrapped)
    );
    assert!(transition.new_state().readiness_exited());
    assert_eq!(transition.new_state().collecting_confirmed_count(), 4);
    assert_eq!(
        transition.outputs(),
        &[
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::ReadyQuorumReached,
            Outcome::ReadinessExited {
                mode: ReadinessExitMode::Bootstrapped
            },
        ]
    );
}

#[test]
fn apply_observes_deadline_exit_transition() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.process(Command::LocalParticipationCompleted);
    let input = Command::DeadlineExpired;

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 3);
    let (observed_input, transition) = &obs[2];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.previous_state().node_state(),
        NodeState::Collecting
    );
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(transition.new_state().node_state(), NodeState::TimedOut);
    assert_eq!(
        transition.new_state().exit_mode(),
        Some(ReadinessExitMode::TimedOut)
    );
    assert!(transition.new_state().readiness_exited());
    assert_eq!(
        transition.outputs(),
        &[Outcome::ReadinessExited {
            mode: ReadinessExitMode::TimedOut
        }]
    );
}

#[test]
fn accepted_delayed_input_is_observable_as_delayed() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();

    // Act
    let outcomes_0 = match coordinator.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 8,
        current_marker: 10,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let outcomes_1 = match coordinator.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let outcomes_2 = match coordinator.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: 8,
        current_marker: 10,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 3);

    let (_, transition0) = &obs[0];
    assert_eq!(
        transition0.outputs(),
        &[Outcome::DelayedParticipationAccepted { peer_id: 1 }]
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
        &[Outcome::DelayedReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(&outcomes_2, transition2.outputs());
}

#[test]
fn state_transition_outputs_are_fully_observable() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();

    // Act
    let outcomes_0 = match coordinator.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let outcomes_1 = match coordinator.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let outcomes_2 = match coordinator.process(Command::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let outcomes_3 = match coordinator.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    let outcomes_4 = match coordinator.process(Command::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
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
            Outcome::ReadyQuorumReached,
            Outcome::ReadinessExited {
                mode: ReadinessExitMode::Bootstrapped,
            },
        ]
    );
    assert_eq!(&outcomes_4, transition4.outputs());
    assert!(transition4.new_state().readiness_exited());
}

#[test]
fn apply_observes_stale_participation_from_initial() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let input = Command::ParticipationObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    };

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 1);
    let (observed_input, transition) = &obs[0];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::StaleParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().pinging_confirmed_count(), 0);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_non_member_participation_from_initial() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let input = Command::ParticipationObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 1);
    let (observed_input, transition) = &obs[0];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().pinging_confirmed_count(), 0);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_stale_ready_from_initial() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let input = Command::ReadyObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    };

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 1);
    let (observed_input, transition) = &obs[0];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::StaleReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().collecting_confirmed_count(), 0);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_non_member_ready_from_initial() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let input = Command::ReadyObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 1);
    let (observed_input, transition) = &obs[0];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().collecting_confirmed_count(), 0);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_duplicate_ready_from_pinging() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.process(Command::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = Command::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 2);
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().collecting_confirmed_count(), 1);
    assert_eq!(transition.new_state().node_state(), NodeState::Pinging);
}

#[test]
fn apply_observes_quorum_exit_from_pinging() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.process(Command::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.process(Command::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    });
    let input = Command::LocalParticipationCompleted;

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 4);
    let (observed_input, transition) = &obs[3];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
            Outcome::ReadyQuorumReached,
            Outcome::ReadinessExited {
                mode: ReadinessExitMode::Bootstrapped,
            },
        ]
    );
    assert_eq!(transition.previous_state().node_state(), NodeState::Pinging);
    assert!(!transition.previous_state().is_pinging_completed());
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(transition.new_state().node_state(), NodeState::Bootstrapped);
    assert_eq!(
        transition.new_state().exit_mode(),
        Some(ReadinessExitMode::Bootstrapped)
    );
    assert!(transition.new_state().readiness_exited());
    assert_eq!(transition.new_state().collecting_confirmed_count(), 4);
}

#[test]
fn apply_observes_deadline_exit_from_pinging() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = Command::DeadlineExpired;

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 2);
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(transition.previous_state().node_state(), NodeState::Pinging);
    assert!(!transition.previous_state().is_pinging_completed());
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(
        transition.outputs(),
        &[Outcome::ReadinessExited {
            mode: ReadinessExitMode::TimedOut
        }]
    );
    assert_eq!(transition.new_state().node_state(), NodeState::TimedOut);
    assert_eq!(
        transition.new_state().exit_mode(),
        Some(ReadinessExitMode::TimedOut)
    );
    assert!(transition.new_state().readiness_exited());
    assert!(!transition.new_state().is_pinging_completed());
}

#[test]
fn apply_observes_timely_ready_from_collecting_no_quorum() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.process(Command::LocalParticipationCompleted);
    let _ = coordinator.process(Command::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = Command::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 4);
    let (observed_input, transition) = &obs[3];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::ReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(
        transition.previous_state().node_state(),
        NodeState::Collecting
    );
    assert_eq!(transition.previous_state().collecting_confirmed_count(), 2);
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(transition.new_state().node_state(), NodeState::Collecting);
    assert_eq!(transition.new_state().collecting_confirmed_count(), 3);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_duplicate_ready_from_collecting() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.process(Command::LocalParticipationCompleted);
    let _ = coordinator.process(Command::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = Command::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 4);
    let (observed_input, transition) = &obs[3];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[Outcome::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().node_state(), NodeState::Collecting);
    assert_eq!(transition.new_state().collecting_confirmed_count(), 2);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_delayed_quorum_exit_from_collecting() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.process(Command::LocalParticipationCompleted);
    let _ = coordinator.process(Command::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let input = Command::ReadyObserved {
        peer_id: 3,
        freshness: 8,
        current_marker: 10,
    };

    // Act
    let outcomes = match coordinator.process(input) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 5);
    let (observed_input, transition) = &obs[4];
    assert_eq!(*observed_input, input);
    assert_eq!(&outcomes, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[
            Outcome::DelayedReadyAccepted { peer_id: 3 },
            Outcome::ReadyQuorumReached,
            Outcome::ReadinessExited {
                mode: ReadinessExitMode::Bootstrapped,
            },
        ]
    );
    assert_eq!(
        transition.previous_state().node_state(),
        NodeState::Collecting
    );
    assert_eq!(transition.previous_state().collecting_confirmed_count(), 3);
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(transition.new_state().node_state(), NodeState::Bootstrapped);
    assert_eq!(
        transition.new_state().exit_mode(),
        Some(ReadinessExitMode::Bootstrapped)
    );
    assert!(transition.new_state().readiness_exited());
    assert_eq!(transition.new_state().collecting_confirmed_count(), 4);
}
