// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::freshness_policy::FreshnessPolicy;
use faction::machine::Machine;
use faction::machine_config::MachineConfig;
use faction::machine_input::MachineInput;
use faction::machine_output::MachineOutput;
use faction::no_op_machine_observer::NoOpMachineObserver;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;

const PEER_SET: &[u64] = &[0, 1, 2, 3, 4];
const THRESHOLD: usize = 4;
const MAX_DELAY: u64 = 2;
const MARKER: u64 = 10;
const TIMELY: u64 = 10;
const DELAYED: u64 = 8;
const STALE: u64 = 7;

fn machine_in_phase1() -> Machine {
    let mut machine = Machine::new(
        MachineConfig::new(
            0,
            PEER_SET.to_vec(),
            QuorumPolicy::new(THRESHOLD),
            FreshnessPolicy::new(MAX_DELAY),
        ),
        Box::new(NoOpMachineObserver),
    );
    let _ = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    machine
}

fn p1(machine: &Machine) -> usize {
    machine.snapshot().phase1_confirmed_count()
}
fn p2(machine: &Machine) -> usize {
    machine.snapshot().phase2_confirmed_count()
}

#[test]
fn deal_accepts_participation_observed() {
    // Arrange
    let mut machine = machine_in_phase1();

    // Act
    let result = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });

    // Assert
    assert_eq!(
        result,
        vec![MachineOutput::ParticipationAccepted { peer_id: 2 }]
    );
}

#[test]
fn deal_accepts_ready_observed() {
    let mut machine = machine_in_phase1();
    let result = machine.apply(MachineInput::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![MachineOutput::ReadyAccepted { peer_id: 2 }]);
}

#[test]
fn deal_accepts_local_participation_completed() {
    let mut machine = machine_in_phase1();
    let result = machine.apply(MachineInput::LocalParticipationCompleted);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], MachineOutput::LocalParticipationCompleted);
    assert_eq!(result[1], MachineOutput::BroadcastLocalReady);
}

#[test]
fn deal_accepts_deadline_expired() {
    let mut machine = machine_in_phase1();
    let result = machine.apply(MachineInput::DeadlineExpired);
    assert_eq!(
        result,
        vec![MachineOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline
        }]
    );
}

#[test]
fn participation_observed_non_member() {
    let mut machine = machine_in_phase1();
    let before = p1(&machine);
    let result = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 99,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![MachineOutput::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(p1(&machine), before);
}

#[test]
fn participation_observed_stale() {
    let mut machine = machine_in_phase1();
    let before = p1(&machine);
    let result = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 2,
        freshness: STALE,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![MachineOutput::StaleParticipationIgnored { peer_id: 2 }]
    );
    assert_eq!(p1(&machine), before);
}

#[test]
fn participation_observed_duplicate() {
    let mut machine = machine_in_phase1();
    let _ = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let before = p1(&machine);
    let result = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![MachineOutput::DuplicateParticipationIgnored { peer_id: 2 }]
    );
    assert_eq!(p1(&machine), before);
}

#[test]
fn participation_observed_first_timely() {
    let mut machine = machine_in_phase1();
    let before = p1(&machine);
    let result = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![MachineOutput::ParticipationAccepted { peer_id: 3 }]
    );
    assert_eq!(p1(&machine), before + 1);
}

#[test]
fn participation_observed_first_delayed() {
    let mut machine = machine_in_phase1();
    let before = p1(&machine);
    let result = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 3,
        freshness: DELAYED,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![MachineOutput::DelayedParticipationAccepted { peer_id: 3 }]
    );
    assert_eq!(p1(&machine), before + 1);
}

#[test]
fn ready_observed_non_member() {
    let mut machine = machine_in_phase1();
    let before = p2(&machine);
    let result = machine.apply(MachineInput::ReadyObserved {
        peer_id: 99,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![MachineOutput::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(p2(&machine), before);
}

#[test]
fn ready_observed_stale() {
    let mut machine = machine_in_phase1();
    let before = p2(&machine);
    let result = machine.apply(MachineInput::ReadyObserved {
        peer_id: 2,
        freshness: STALE,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![MachineOutput::StaleReadyIgnored { peer_id: 2 }]
    );
    assert_eq!(p2(&machine), before);
}

#[test]
fn ready_observed_duplicate() {
    let mut machine = machine_in_phase1();
    let _ = machine.apply(MachineInput::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let before = p2(&machine);
    let result = machine.apply(MachineInput::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![MachineOutput::DuplicateReadyIgnored { peer_id: 2 }]
    );
    assert_eq!(p2(&machine), before);
}

#[test]
fn ready_observed_first_timely() {
    let mut machine = machine_in_phase1();
    let before = p2(&machine);
    let result = machine.apply(MachineInput::ReadyObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![MachineOutput::ReadyAccepted { peer_id: 3 }]);
    assert_eq!(p2(&machine), before + 1);
}

#[test]
fn ready_observed_first_delayed() {
    let mut machine = machine_in_phase1();
    let before = p2(&machine);
    let result = machine.apply(MachineInput::ReadyObserved {
        peer_id: 3,
        freshness: DELAYED,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![MachineOutput::DelayedReadyAccepted { peer_id: 3 }]
    );
    assert_eq!(p2(&machine), before + 1);
}

#[test]
fn local_completion_no_quorum() {
    let mut machine = machine_in_phase1();
    let result = machine.apply(MachineInput::LocalParticipationCompleted);
    // Arrange & Act
    assert_eq!(
        result,
        vec![
            MachineOutput::LocalParticipationCompleted,
            MachineOutput::BroadcastLocalReady,
        ]
    );
    // Assert
    let snap = machine.snapshot();
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(snap.local_participation_complete());
    assert!(!snap.readiness_exited());
}

#[test]
fn local_completion_triggers_quorum() {
    // Arrange
    let mut machine = Machine::new(
        MachineConfig::new(
            0,
            PEER_SET.to_vec(),
            QuorumPolicy::new(4),
            FreshnessPolicy::new(MAX_DELAY),
        ),
        Box::new(NoOpMachineObserver),
    );
    let _ = machine.apply(MachineInput::ParticipationObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = machine.apply(MachineInput::ReadyObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = machine.apply(MachineInput::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = machine.apply(MachineInput::ReadyObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    });

    // Act
    let result = machine.apply(MachineInput::LocalParticipationCompleted);

    // Assert
    assert_eq!(
        result,
        vec![
            MachineOutput::LocalParticipationCompleted,
            MachineOutput::BroadcastLocalReady,
            MachineOutput::ReadyQuorumReached,
            MachineOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    let snap = machine.snapshot();
    assert!(snap.readiness_exited());
    assert_eq!(snap.exit_mode(), Some(ReadinessExitMode::Quorum));
}

#[test]
fn deadline_expired_in_phase1() {
    let mut machine = machine_in_phase1();
    // Act & Assert
    let result = machine.apply(MachineInput::DeadlineExpired);
    assert_eq!(
        result,
        vec![MachineOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline,
        }]
    );
    let snap = machine.snapshot();
    assert!(snap.readiness_exited());
    assert_eq!(snap.exit_mode(), Some(ReadinessExitMode::Deadline));
}

#[test]
fn vibe_check_in_phase1() {
    // Arrange & Act
    let machine = machine_in_phase1();
    let snap = machine.snapshot();

    // Assert
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert!(!snap.readiness_exited());
    assert!(!snap.local_participation_complete());
    assert_eq!(snap.exit_mode(), None);
    assert_eq!(snap.phase1_confirmed_count(), 1);
    assert_eq!(snap.phase2_confirmed_count(), 0);
    assert_eq!(snap.quorum_threshold(), THRESHOLD);
}
