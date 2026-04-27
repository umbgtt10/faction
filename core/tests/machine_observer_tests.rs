// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

use faction::freshness_policy::FreshnessPolicy;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::machine::Machine;
use faction::machine_config::MachineConfig;
use faction::machine_input::MachineInput;
use faction::machine_observer::MachineObserver;
use faction::machine_output::MachineOutput;
use faction::machine_transition::MachineTransition;

type Observations = Rc<RefCell<Vec<(MachineInput, MachineTransition)>>>;

struct RecordingMachineObserver {
    observations: Observations,
}

impl MachineObserver for RecordingMachineObserver {
    fn observe(&mut self, input: MachineInput, transition: MachineTransition) {
        self.observations.borrow_mut().push((input, transition));
    }
}

fn recording_coordinator() -> (Machine, Observations) {
    let observations: Observations = Rc::new(RefCell::new(Vec::new()));
    let observer = RecordingMachineObserver {
        observations: Rc::clone(&observations),
    };
    let coordinator = Machine::new(
        MachineConfig::new(
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
    let _ = coordinator.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = MachineInput::LocalParticipationCompleted;

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 2);
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(&outputs, transition.outputs());
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
            MachineOutput::LocalParticipationCompleted,
            MachineOutput::BroadcastLocalReady,
        ]
    );
}

#[test]
fn apply_observes_duplicate_participation_transition_without_state_change() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = MachineInput::ParticipationObserved {
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
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[MachineOutput::DuplicateParticipationIgnored { peer_id: 1 }]
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
    let _ = coordinator.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(MachineInput::LocalParticipationCompleted);
    let input = MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    };

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 3);
    let (observed_input, transition) = &obs[2];
    assert_eq!(*observed_input, input);
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[MachineOutput::StaleReadyIgnored { peer_id: 1 }]
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
    let _ = coordinator.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(MachineInput::LocalParticipationCompleted);
    let _ = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let input = MachineInput::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 5);
    let (observed_input, transition) = &obs[4];
    assert_eq!(*observed_input, input);
    assert_eq!(&outputs, transition.outputs());
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
            MachineOutput::ReadyAccepted { peer_id: 3 },
            MachineOutput::ReadyQuorumReached,
            MachineOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum
            },
        ]
    );
}

#[test]
fn apply_observes_deadline_exit_transition() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(MachineInput::LocalParticipationCompleted);
    let input = MachineInput::DeadlineExpired;

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 3);
    let (observed_input, transition) = &obs[2];
    assert_eq!(*observed_input, input);
    assert_eq!(&outputs, transition.outputs());
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
        &[MachineOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline
        }]
    );
}

#[test]
fn accepted_delayed_input_is_observable_as_delayed() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();

    // Act
    let batch0 = coordinator.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 8,
        current_marker: 10,
    });
    let batch1 = coordinator.apply(MachineInput::LocalParticipationCompleted);
    let batch2 = coordinator.apply(MachineInput::ReadyObserved {
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
        &[MachineOutput::DelayedParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(&batch0, transition0.outputs());

    let (_, transition1) = &obs[1];
    assert_eq!(
        transition1.outputs(),
        &[
            MachineOutput::LocalParticipationCompleted,
            MachineOutput::BroadcastLocalReady,
        ]
    );
    assert_eq!(&batch1, transition1.outputs());

    let (_, transition2) = &obs[2];
    assert_eq!(
        transition2.outputs(),
        &[MachineOutput::DelayedReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(&batch2, transition2.outputs());
}

#[test]
fn state_transition_outputs_are_fully_observable() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();

    // Act
    let batch0 = coordinator.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let batch1 = coordinator.apply(MachineInput::LocalParticipationCompleted);
    let batch2 = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let batch3 = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let batch4 = coordinator.apply(MachineInput::ReadyObserved {
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
        &[MachineOutput::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(&batch0, transition0.outputs());

    let (_, transition1) = &obs[1];
    assert_eq!(
        transition1.outputs(),
        &[
            MachineOutput::LocalParticipationCompleted,
            MachineOutput::BroadcastLocalReady,
        ]
    );
    assert_eq!(&batch1, transition1.outputs());

    let (_, transition2) = &obs[2];
    assert_eq!(
        transition2.outputs(),
        &[MachineOutput::ReadyAccepted { peer_id: 1 }]
    );
    assert_eq!(&batch2, transition2.outputs());

    let (_, transition3) = &obs[3];
    assert_eq!(
        transition3.outputs(),
        &[MachineOutput::ReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(&batch3, transition3.outputs());

    let (_, transition4) = &obs[4];
    assert_eq!(
        transition4.outputs(),
        &[
            MachineOutput::ReadyAccepted { peer_id: 3 },
            MachineOutput::ReadyQuorumReached,
            MachineOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(&batch4, transition4.outputs());
    assert!(transition4.new_state().readiness_exited());
}

#[test]
fn apply_observes_stale_participation_from_initial() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let input = MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    };

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 1);
    let (observed_input, transition) = &obs[0];
    assert_eq!(*observed_input, input);
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[MachineOutput::StaleParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().phase1_confirmed_count(), 0);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_non_member_participation_from_initial() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let input = MachineInput::ParticipationObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 1);
    let (observed_input, transition) = &obs[0];
    assert_eq!(*observed_input, input);
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[MachineOutput::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().phase1_confirmed_count(), 0);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_stale_ready_from_initial() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let input = MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    };

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 1);
    let (observed_input, transition) = &obs[0];
    assert_eq!(*observed_input, input);
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[MachineOutput::StaleReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().phase2_confirmed_count(), 0);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_non_member_ready_from_initial() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let input = MachineInput::ReadyObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    };

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 1);
    let (observed_input, transition) = &obs[0];
    assert_eq!(*observed_input, input);
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[MachineOutput::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().phase2_confirmed_count(), 0);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_duplicate_ready_from_pinging() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = MachineInput::ReadyObserved {
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
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[MachineOutput::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(transition.new_state().phase2_confirmed_count(), 1);
    assert_eq!(
        transition.new_state().lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
}

#[test]
fn apply_observes_quorum_exit_from_pinging() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    });
    let input = MachineInput::LocalParticipationCompleted;

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 4);
    let (observed_input, transition) = &obs[3];
    assert_eq!(*observed_input, input);
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[
            MachineOutput::LocalParticipationCompleted,
            MachineOutput::BroadcastLocalReady,
            MachineOutput::ReadyQuorumReached,
            MachineOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(
        transition.previous_state().lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert!(!transition.previous_state().local_participation_complete());
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
}

#[test]
fn apply_observes_deadline_exit_from_pinging() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = MachineInput::DeadlineExpired;

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 2);
    let (observed_input, transition) = &obs[1];
    assert_eq!(*observed_input, input);
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.previous_state().lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert!(!transition.previous_state().local_participation_complete());
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(
        transition.outputs(),
        &[MachineOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline
        }]
    );
    assert_eq!(
        transition.new_state().lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert_eq!(
        transition.new_state().exit_mode(),
        Some(ReadinessExitMode::Deadline)
    );
    assert!(transition.new_state().readiness_exited());
    assert!(!transition.new_state().local_participation_complete());
}

#[test]
fn apply_observes_timely_ready_from_collecting_no_quorum() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(MachineInput::LocalParticipationCompleted);
    let _ = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = MachineInput::ReadyObserved {
        peer_id: 2,
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
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[MachineOutput::ReadyAccepted { peer_id: 2 }]
    );
    assert_eq!(
        transition.previous_state().lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(transition.previous_state().phase2_confirmed_count(), 2);
    assert!(!transition.previous_state().readiness_exited());
    assert_eq!(
        transition.new_state().lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(transition.new_state().phase2_confirmed_count(), 3);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_duplicate_ready_from_collecting() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(MachineInput::LocalParticipationCompleted);
    let _ = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let input = MachineInput::ReadyObserved {
        peer_id: 1,
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
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[MachineOutput::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(transition.previous_state(), transition.new_state());
    assert_eq!(
        transition.new_state().lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(transition.new_state().phase2_confirmed_count(), 2);
    assert!(!transition.new_state().readiness_exited());
}

#[test]
fn apply_observes_delayed_quorum_exit_from_collecting() {
    // Arrange
    let (mut coordinator, observations) = recording_coordinator();
    let _ = coordinator.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(MachineInput::LocalParticipationCompleted);
    let _ = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(MachineInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let input = MachineInput::ReadyObserved {
        peer_id: 3,
        freshness: 8,
        current_marker: 10,
    };

    // Act
    let outputs = coordinator.apply(input);

    // Assert
    let obs = observations.borrow();
    assert_eq!(obs.len(), 5);
    let (observed_input, transition) = &obs[4];
    assert_eq!(*observed_input, input);
    assert_eq!(&outputs, transition.outputs());
    assert_eq!(
        transition.outputs(),
        &[
            MachineOutput::DelayedReadyAccepted { peer_id: 3 },
            MachineOutput::ReadyQuorumReached,
            MachineOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
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
}
