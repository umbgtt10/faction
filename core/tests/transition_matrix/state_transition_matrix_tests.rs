// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_machine_observer::NoOpMachineObserver;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::machine::Machine;
use faction::machine_config::MachineConfig;
use faction::machine_input::MachineInput;
use faction::machine_output::MachineOutput;
use faction::machine_output::MachineOutput::*;
use faction::Freshness;
use faction::PeerId;
use rstest::rstest;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const THRESHOLD: usize = 5;
const MAX_DELAY: Freshness = 2;
const MARKER: Freshness = 10;
const TIMELY: Freshness = 10;
const DELAYED: Freshness = 8;
const STALE: Freshness = 7;

// ---------------------------------------------------------------------------
// Init — which logical state to start from
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Init {
    Fresh,
    Phase1Peer1Confirmed,
    Phase1P2Threshold,
    Phase2NoReadiness,
    Phase2Peer1Confirmed,
    Phase2AlmostQuorum,
}

fn build(init: Init) -> Machine {
    let mut m = Machine::new(
        MachineConfig::new(
            0,
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(THRESHOLD),
            FreshnessPolicy::new(MAX_DELAY),
        ),
        Box::new(NoOpMachineObserver),
    );
    let _ = m.apply(MachineInput::ParticipationObserved {
        peer_id: 99,
        freshness: MARKER,
        current_marker: MARKER,
    });
    match init {
        Init::Fresh => {}
        Init::Phase1Peer1Confirmed => {
            let _ = m.apply(MachineInput::ParticipationObserved {
                peer_id: 1,
                freshness: TIMELY,
                current_marker: MARKER,
            });
        }
        Init::Phase1P2Threshold => {
            for peer in 0..5 {
                let _ = m.apply(MachineInput::ReadyObserved {
                    peer_id: peer,
                    freshness: TIMELY,
                    current_marker: MARKER,
                });
            }
        }
        Init::Phase2NoReadiness => {
            let _ = m.apply(MachineInput::LocalParticipationCompleted);
        }
        Init::Phase2Peer1Confirmed => {
            let _ = m.apply(MachineInput::LocalParticipationCompleted);
            let _ = m.apply(MachineInput::ReadyObserved {
                peer_id: 1,
                freshness: TIMELY,
                current_marker: MARKER,
            });
        }
        Init::Phase2AlmostQuorum => {
            let _ = m.apply(MachineInput::LocalParticipationCompleted);
            for peer in 1..4 {
                let _ = m.apply(MachineInput::ReadyObserved {
                    peer_id: peer,
                    freshness: TIMELY,
                    current_marker: MARKER,
                });
            }
        }
    }
    m
}

// ---------------------------------------------------------------------------
// Assert — what to verify about the resulting state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Assert {
    P1Count(usize),
    P2Count(usize),
    Exited,
    NotExited,
    ExitMode(ReadinessExitMode),
    LocalComplete,
    NotLocalComplete,
}

fn verify(m: &Machine, checks: &[Assert]) {
    let s = m.snapshot();
    for check in checks {
        match *check {
            Assert::P1Count(n) => assert_eq!(s.phase1_confirmed_count(), n),
            Assert::P2Count(n) => assert_eq!(s.phase2_confirmed_count(), n),
            Assert::Exited => assert!(s.readiness_exited()),
            Assert::NotExited => assert!(!s.readiness_exited()),
            Assert::ExitMode(mode) => assert_eq!(s.exit_mode(), Some(mode)),
            Assert::LocalComplete => assert!(s.local_participation_complete()),
            Assert::NotLocalComplete => assert!(!s.local_participation_complete()),
        }
    }
}

// ---------------------------------------------------------------------------
// Input helpers
// ---------------------------------------------------------------------------

fn participation(peer_id: PeerId, freshness: Freshness) -> MachineInput {
    MachineInput::ParticipationObserved {
        peer_id,
        freshness,
        current_marker: MARKER,
    }
}

fn ready(peer_id: PeerId, freshness: Freshness) -> MachineInput {
    MachineInput::ReadyObserved {
        peer_id,
        freshness,
        current_marker: MARKER,
    }
}

// ---------------------------------------------------------------------------
// Single rstest — every valid (state × input) pair
// ---------------------------------------------------------------------------

#[rstest]
#[case::participation_timely_member(
    Init::Fresh,
    participation(1, TIMELY),
    &[ParticipationAccepted { peer_id: 1 }],
    &[Assert::P1Count(1), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::participation_delayed_member(
    Init::Fresh,
    participation(1, DELAYED),
    &[DelayedParticipationAccepted { peer_id: 1 }],
    &[Assert::P1Count(1), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::participation_stale_member(
    Init::Fresh,
    participation(1, STALE),
    &[StaleParticipationIgnored { peer_id: 1 }],
    &[Assert::P1Count(0), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::participation_non_member(
    Init::Fresh,
    participation(99, TIMELY),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::P1Count(0), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::participation_duplicate(
    Init::Phase1Peer1Confirmed,
    participation(1, TIMELY),
    &[DuplicateParticipationIgnored { peer_id: 1 }],
    &[Assert::P1Count(1), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::ready_timely_member(
    Init::Fresh,
    ready(1, TIMELY),
    &[ReadyAccepted { peer_id: 1 }],
    &[Assert::P2Count(1), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::ready_delayed_member(
    Init::Fresh,
    ready(1, DELAYED),
    &[DelayedReadyAccepted { peer_id: 1 }],
    &[Assert::P2Count(1), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::ready_stale_member(
    Init::Fresh,
    ready(1, STALE),
    &[StaleReadyIgnored { peer_id: 1 }],
    &[Assert::P2Count(0), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::ready_non_member(
    Init::Fresh,
    ready(99, TIMELY),
    &[NonMemberIgnored { peer_id: 99 }],
    &[Assert::P2Count(0), Assert::NotExited, Assert::NotLocalComplete],
)]
#[case::ready_duplicate(
    Init::Phase2Peer1Confirmed,
    ready(1, TIMELY),
    &[DuplicateReadyIgnored { peer_id: 1 }],
    &[Assert::P2Count(2), Assert::NotExited, Assert::LocalComplete],
)]
#[case::ready_timely_triggers_quorum(
    Init::Phase2AlmostQuorum,
    ready(4, TIMELY),
    &[
        ReadyAccepted { peer_id: 4 },
        ReadyQuorumReached,
        ReadinessExited { mode: ReadinessExitMode::Quorum },
    ],
    &[Assert::P2Count(5), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Quorum)],
)]
#[case::ready_delayed_triggers_quorum(
    Init::Phase2AlmostQuorum,
    ready(4, DELAYED),
    &[
        DelayedReadyAccepted { peer_id: 4 },
        ReadyQuorumReached,
        ReadinessExited { mode: ReadinessExitMode::Quorum },
    ],
    &[Assert::P2Count(5), Assert::Exited, Assert::ExitMode(ReadinessExitMode::Quorum)],
)]
#[case::local_completion_transitions_to_phase2(
    Init::Fresh,
    MachineInput::LocalParticipationCompleted,
    &[LocalParticipationCompleted, BroadcastLocalReady],
    &[Assert::P2Count(1), Assert::LocalComplete, Assert::NotExited],
)]
#[case::local_completion_with_preloaded_quorum(
    Init::Phase1P2Threshold,
    MachineInput::LocalParticipationCompleted,
    &[
        LocalParticipationCompleted,
        BroadcastLocalReady,
        ReadyQuorumReached,
        ReadinessExited { mode: ReadinessExitMode::Quorum },
    ],
    &[Assert::P2Count(5), Assert::LocalComplete, Assert::Exited, Assert::ExitMode(ReadinessExitMode::Quorum)],
)]
#[case::local_completion_redundant(
    Init::Phase2NoReadiness,
    MachineInput::LocalParticipationCompleted,
    &[],
    &[Assert::P2Count(1), Assert::LocalComplete, Assert::NotExited],
)]
#[case::deadline_expired(
    Init::Fresh,
    MachineInput::DeadlineExpired,
    &[ReadinessExited { mode: ReadinessExitMode::Deadline }],
    &[Assert::Exited, Assert::ExitMode(ReadinessExitMode::Deadline)],
)]
fn valid_transition(
    #[case] init: Init,
    #[case] input: MachineInput,
    #[case] expected_outputs: &[MachineOutput],
    #[case] asserts: &[Assert],
) {
    // Arrange
    let mut m = build(init);

    // Act
    let batch = m.apply(input);

    // Assert
    assert_eq!(batch.as_slice(), expected_outputs, "output mismatch");
    verify(&m, asserts);
}
