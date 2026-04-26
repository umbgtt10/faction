// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

use faction::cluster_readiness::ClusterReadiness;
use faction::cluster_readiness_config::ClusterReadinessConfig;
use faction::cluster_readiness_input::ClusterReadinessInput;
use faction::cluster_readiness_observer::ClusterReadinessObserver;
use faction::cluster_readiness_output::ClusterReadinessOutput;
use faction::cluster_readiness_transition::ClusterReadinessTransition;
use faction::freshness_policy::FreshnessPolicy;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;

type Observations = Rc<RefCell<Vec<(ClusterReadinessInput, ClusterReadinessTransition)>>>;

struct RecordingClusterReadinessObserver {
    observations: Observations,
}

impl ClusterReadinessObserver for RecordingClusterReadinessObserver {
    fn observe(&mut self, input: ClusterReadinessInput, transition: ClusterReadinessTransition) {
        self.observations.borrow_mut().push((input, transition));
    }
}

fn recording_coordinator() -> (ClusterReadiness, Observations) {
    let observations: Observations = Rc::new(RefCell::new(Vec::new()));
    let observer = RecordingClusterReadinessObserver {
        observations: Rc::clone(&observations),
    };
    let coordinator = ClusterReadiness::new(
        ClusterReadinessConfig::new(
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
    let input = ClusterReadinessInput::LocalParticipationCompleted;

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 1);
    let (observed_input, transition) = &obs[0];
    assert_eq!(*observed_input, input);
    assert_eq!(outputs.outputs(), transition.outputs());
    assert_eq!(
        transition.previous_state().lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert!(!transition.previous_state().local_participation_complete());
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(
        transition.new_state().lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(transition.new_state().local_participation_complete());
    assert!(!transition.new_state().readiness_exited());
    assert_eq!(transition.new_state().phase2_confirmed_count(), 1);
    assert_eq!(
        transition.outputs(),
        &[
            ClusterReadinessOutput::LocalParticipationCompleted,
            ClusterReadinessOutput::BroadcastLocalReady,
        ]
    );
}

#[test]
fn apply_observes_duplicate_participation_transition_without_state_change() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 2);
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(outputs.outputs(), transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[ClusterReadinessOutput::DuplicateParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().phase1_confirmed_count(), 1);
    assert_eq!(
        transition.new_state().lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
}

#[test]
fn apply_observes_stale_ready_transition_without_state_change() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let input = ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    };

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 2);
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(outputs.outputs(), transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[ClusterReadinessOutput::StaleReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(
        transition.new_state().lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(transition.new_state().phase2_confirmed_count(), 1);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_quorum_exit_transition() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let input = ClusterReadinessInput::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 4);
    let (observed_input, transition) = &obs[3];
    assert_eq!(*observed_input, input);
    assert_eq!(outputs.outputs(), transition.outputs());
    assert_eq!(
        transition.previous_state().lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(transition.previous_state().phase2_confirmed_count(), 3);
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(
        transition.new_state().lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert_eq!(
        transition.new_state().exit_mode(),
        Some(ReadinessExitMode::Quorum)
    );
    assert!(transition.new_state().readiness_exited());
    assert_eq!(transition.new_state().phase2_confirmed_count(), 4);
    assert_eq!(
        transition.outputs(),
        &[
            ClusterReadinessOutput::ReadyAccepted { peer_id: 3 },
            ClusterReadinessOutput::ReadyQuorumReached,
            ClusterReadinessOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum
            },
        ]
    );
}

#[test]
fn apply_observes_deadline_exit_transition() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let input = ClusterReadinessInput::DeadlineExpired;

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 2);
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(outputs.outputs(), transition.outputs());
    assert_eq!(
        transition.previous_state().lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(
        transition.new_state().lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert_eq!(
        transition.new_state().exit_mode(),
        Some(ReadinessExitMode::Deadline)
    );
    assert!(transition.new_state().readiness_exited());
    assert_eq!(
        transition.outputs(),
        &[ClusterReadinessOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline
        }]
    );
}

#[test]
fn accepted_delayed_input_is_observable_as_delayed() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();

    // Act
    let batch0 = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 8,
        current_marker: 10,
    });
    let batch1 = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let batch2 = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 2,
        freshness: 8,
        current_marker: 10,
    });

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 3);

    let (_, transition0) = &obs[0];
    assert_eq!(
        transition0.outputs(),
        &[ClusterReadinessOutput::DelayedParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(batch0.outputs(), transition0.outputs());

    let (_, transition1) = &obs[1];
    assert_eq!(
        transition1.outputs(),
        &[
            ClusterReadinessOutput::LocalParticipationCompleted,
            ClusterReadinessOutput::BroadcastLocalReady,
        ]
    );
    assert_eq!(batch1.outputs(), transition1.outputs());

    let (_, transition2) = &obs[2];
    assert_eq!(
        transition2.outputs(),
        &[ClusterReadinessOutput::DelayedReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(batch2.outputs(), transition2.outputs());
}

#[test]
fn state_transition_outputs_are_fully_observable() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();

    // Act
    let batch0 = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let batch1 = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let batch2 = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let batch3 = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let batch4 = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    });

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 5);

    let (_, transition0) = &obs[0];
    assert_eq!(
        transition0.outputs(),
        &[ClusterReadinessOutput::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(batch0.outputs(), transition0.outputs());

    let (_, transition1) = &obs[1];
    assert_eq!(
        transition1.outputs(),
        &[
            ClusterReadinessOutput::LocalParticipationCompleted,
            ClusterReadinessOutput::BroadcastLocalReady,
        ]
    );
    assert_eq!(batch1.outputs(), transition1.outputs());

    let (_, transition2) = &obs[2];
    assert_eq!(
        transition2.outputs(),
        &[ClusterReadinessOutput::ReadyAccepted { peer_id: 1 }]
    );
    assert_eq!(batch2.outputs(), transition2.outputs());

    let (_, transition3) = &obs[3];
    assert_eq!(
        transition3.outputs(),
        &[ClusterReadinessOutput::ReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(batch3.outputs(), transition3.outputs());

    let (_, transition4) = &obs[4];
    assert_eq!(
        transition4.outputs(),
        &[
            ClusterReadinessOutput::ReadyAccepted { peer_id: 3 },
            ClusterReadinessOutput::ReadyQuorumReached,
            ClusterReadinessOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(batch4.outputs(), transition4.outputs());
    assert!(transition4.new_state().readiness_exited());
}
