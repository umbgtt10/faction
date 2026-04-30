// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::node_state::NodeState;
use proptest::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelLifecycleState {
    Pinging,
    Collecting,
    Bootstrapped,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ModelClusterView {
    node_state: ModelLifecycleState,
    exit_mode: Option<ReadinessExitMode>,
    local_participation_complete: bool,
    readiness_exited: bool,
    pinging_confirmed_count: usize,
    collecting_confirmed_count: usize,
    required_count: usize,
}

struct ModelCoordinator {
    local_peer_id: u64,
    peer_set: [u64; 5],
    required_count: usize,
    max_delay: u64,
    initial: bool,
    node_state: ModelLifecycleState,
    exit_mode: Option<ReadinessExitMode>,
    local_participation_complete: bool,
    phase1_confirmed: [bool; 5],
    phase2_confirmed: [bool; 5],
    pinging_confirmed_count: usize,
    collecting_confirmed_count: usize,
}

impl ModelCoordinator {
    fn new() -> Self {
        Self {
            local_peer_id: 0,
            peer_set: [0, 1, 2, 3, 4],
            required_count: 4,
            max_delay: 2,
            initial: true,
            node_state: ModelLifecycleState::Pinging,
            exit_mode: None,
            local_participation_complete: false,
            phase1_confirmed: [false; 5],
            phase2_confirmed: [false; 5],
            pinging_confirmed_count: 0,
            collecting_confirmed_count: 0,
        }
    }

    fn cluster_view(&self) -> ModelClusterView {
        ModelClusterView {
            node_state: self.node_state,
            exit_mode: self.exit_mode,
            local_participation_complete: self.local_participation_complete,
            readiness_exited: self.exit_mode.is_some(),
            pinging_confirmed_count: self.pinging_confirmed_count,
            collecting_confirmed_count: self.collecting_confirmed_count,
            required_count: self.required_count,
        }
    }

    fn process(&mut self, input: Command) -> alloc::vec::Vec<Outcome> {
        if self.initial {
            match input {
                Command::ParticipationObserved { .. } | Command::ReadyObserved { .. } => {
                    self.initial = false;
                }
                _ => return Vec::new(),
            }
        }

        if self.has_exited() {
            return Vec::new();
        }

        if self.local_participation_complete {
            match input {
                Command::ParticipationObserved { .. } | Command::LocalParticipationCompleted => {
                    return Vec::new()
                }
                _ => {}
            }
        }

        match input {
            Command::ParticipationObserved {
                peer_id,
                freshness,
                current_marker,
            } => self.apply_participation_observed(peer_id, freshness, current_marker),
            Command::ReadyObserved {
                peer_id,
                freshness,
                current_marker,
            } => self.apply_ready_observed(peer_id, freshness, current_marker),
            Command::LocalParticipationCompleted => self.apply_local_participation_completed(),
            Command::DeadlineExpired => self.apply_deadline_expired(),
            Command::Probe => unreachable!("Probe handled in Faction::process"),
        }
    }

    fn apply_participation_observed(
        &mut self,
        peer_id: u64,
        freshness: u64,
        current_marker: u64,
    ) -> alloc::vec::Vec<Outcome> {
        if self.has_exited() {
            return vec![Outcome::StaleParticipationIgnored { peer_id }];
        }

        let Some(index) = self.peer_index(peer_id) else {
            return vec![Outcome::NonMemberIgnored { peer_id }];
        };

        if self.is_stale(current_marker, freshness) {
            return vec![Outcome::StaleParticipationIgnored { peer_id }];
        }

        if self.phase1_confirmed[index] {
            return vec![Outcome::DuplicateParticipationIgnored { peer_id }];
        }

        self.phase1_confirmed[index] = true;
        self.pinging_confirmed_count += 1;

        if self.is_delayed(current_marker, freshness) {
            vec![Outcome::DelayedParticipationAccepted { peer_id }]
        } else {
            vec![Outcome::ParticipationAccepted { peer_id }]
        }
    }

    fn apply_ready_observed(
        &mut self,
        peer_id: u64,
        freshness: u64,
        current_marker: u64,
    ) -> alloc::vec::Vec<Outcome> {
        if self.has_exited() {
            return vec![Outcome::StaleReadyIgnored { peer_id }];
        }

        let Some(index) = self.peer_index(peer_id) else {
            return vec![Outcome::NonMemberIgnored { peer_id }];
        };

        if self.is_stale(current_marker, freshness) {
            return vec![Outcome::StaleReadyIgnored { peer_id }];
        }

        if self.phase2_confirmed[index] {
            return vec![Outcome::DuplicateReadyIgnored { peer_id }];
        }

        self.phase2_confirmed[index] = true;
        self.collecting_confirmed_count += 1;

        let accepted_output = if self.is_delayed(current_marker, freshness) {
            Outcome::DelayedReadyAccepted { peer_id }
        } else {
            Outcome::ReadyAccepted { peer_id }
        };

        if self.local_participation_complete && self.collecting_confirmed_count >= self.required_count
        {
            self.exit_mode = Some(ReadinessExitMode::Bootstrapped);
            self.node_state = ModelLifecycleState::Bootstrapped;
            vec![
                accepted_output,
                Outcome::ReadyQuorumReached,
                Outcome::ReadinessExited {
                    mode: ReadinessExitMode::Bootstrapped,
                },
            ]
        } else {
            vec![accepted_output]
        }
    }

    fn apply_local_participation_completed(&mut self) -> alloc::vec::Vec<Outcome> {
        if self.has_exited() || self.local_participation_complete {
            return vec![];
        }

        self.local_participation_complete = true;
        self.node_state = ModelLifecycleState::Collecting;

        let local_index = self
            .peer_index(self.local_peer_id)
            .expect("local peer must be in peer set");
        if !self.phase2_confirmed[local_index] {
            self.phase2_confirmed[local_index] = true;
            self.collecting_confirmed_count += 1;
        }

        let mut outputs = vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ];

        if self.collecting_confirmed_count >= self.required_count {
            self.exit_mode = Some(ReadinessExitMode::Bootstrapped);
            self.node_state = ModelLifecycleState::Bootstrapped;
            outputs.push(Outcome::ReadyQuorumReached);
            outputs.push(Outcome::ReadinessExited {
                mode: ReadinessExitMode::Bootstrapped,
            });
        }

        outputs
    }

    fn apply_deadline_expired(&mut self) -> alloc::vec::Vec<Outcome> {
        if self.has_exited() {
            return vec![];
        }

        self.exit_mode = Some(ReadinessExitMode::TimedOut);
        self.node_state = ModelLifecycleState::TimedOut;

        vec![Outcome::ReadinessExited {
            mode: ReadinessExitMode::TimedOut,
        }]
    }

    fn peer_index(&self, peer_id: u64) -> Option<usize> {
        self.peer_set
            .iter()
            .position(|candidate| *candidate == peer_id)
    }

    fn has_exited(&self) -> bool {
        self.exit_mode.is_some()
    }

    fn is_stale(&self, current_marker: u64, freshness: u64) -> bool {
        if freshness > current_marker {
            return true;
        }

        current_marker.saturating_sub(freshness) > self.max_delay
    }

    fn is_delayed(&self, current_marker: u64, freshness: u64) -> bool {
        !self.is_stale(current_marker, freshness) && freshness < current_marker
    }
}

fn model_snapshot(cluster_view: ClusterView) -> ModelClusterView {
    ModelClusterView {
        node_state: match cluster_view.node_state() {
            NodeState::Pinging => ModelLifecycleState::Pinging,
            NodeState::Collecting => ModelLifecycleState::Collecting,
            NodeState::Bootstrapped => ModelLifecycleState::Bootstrapped,
            NodeState::TimedOut => ModelLifecycleState::TimedOut,
        },
        exit_mode: cluster_view.exit_mode(),
        local_participation_complete: cluster_view.local_participation_complete(),
        readiness_exited: cluster_view.readiness_exited(),
        pinging_confirmed_count: cluster_view.pinging_confirmed_count(),
        collecting_confirmed_count: cluster_view.collecting_confirmed_count(),
        required_count: cluster_view.required_count(),
    }
}

fn test_config() -> Config {
    Config::new(
        0,
        vec![0, 1, 2, 3, 4],
        QuorumPolicy::new(4),
        FreshnessPolicy::new(2),
    )
}

fn coordinator() -> Faction {
    Faction::new(test_config(), Box::new(NoOpObserver))
}

fn input_strategy() -> impl Strategy<Value = Command> {
    let participation =
        (0u64..=6, 0u64..=12, 0u64..=12).prop_map(|(peer_id, freshness, current_marker)| {
            Command::ParticipationObserved {
                peer_id,
                freshness,
                current_marker,
            }
        });
    let ready =
        (0u64..=6, 0u64..=12, 0u64..=12).prop_map(|(peer_id, freshness, current_marker)| {
            Command::ReadyObserved {
                peer_id,
                freshness,
                current_marker,
            }
        });

    prop_oneof![
        participation,
        ready,
        Just(Command::LocalParticipationCompleted),
        Just(Command::DeadlineExpired),
    ]
}

proptest! {
    #[test]
    fn model_matches_snapshot_and_outputs_for_random_sequences(
        inputs in prop::collection::vec(input_strategy(), 0..64)
    ) {
        // Arrange
        let mut coordinator = coordinator();
        let mut model = ModelCoordinator::new();

        // Act
        for input in inputs {
            let actual_outputs = match coordinator.process(input) {
                ProcessResult::Accepted { outcomes, .. } => outcomes,
                ProcessResult::Probed { .. } => unreachable!(),
                ProcessResult::Rejected { .. } => vec![],
            };
            let actual_snapshot = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };
            let expected_outputs = model.process(input);
            let expected_snapshot = model.cluster_view();

            // Assert
            prop_assert_eq!(actual_outputs.as_slice(), expected_outputs.as_slice());
            prop_assert_eq!(model_snapshot(actual_snapshot), expected_snapshot);
        }
    }
}
