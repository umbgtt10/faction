// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
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
use proptest::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelLifecycleState {
    Fresh,
    Pinging,
    Collecting,
    Bootstrapped,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ModelClusterView {
    peer_state: ModelLifecycleState,
    exit_mode: Option<Conclusion>,
    is_pinging_completed: bool,
    readiness_exited: bool,
    pinging_confirmed_count: usize,
    collecting_confirmed_count: usize,
    required_count: usize,
}

struct ModelCoordinator {
    local_peer_id: u64,
    peer_set: [u64; 5],
    required_count: usize,
    initial: bool,
    peer_state: ModelLifecycleState,
    exit_mode: Option<Conclusion>,
    is_pinging_completed: bool,
    pinging_confirmed: [bool; 5],
    collecting_confirmed: [bool; 5],
    pinging_confirmed_count: usize,
    collecting_confirmed_count: usize,
}

impl ModelCoordinator {
    fn new() -> Self {
        Self {
            local_peer_id: 0,
            peer_set: [0, 1, 2, 3, 4],
            required_count: 4,
            initial: true,
            peer_state: ModelLifecycleState::Fresh,
            exit_mode: None,
            is_pinging_completed: false,
            pinging_confirmed: [false; 5],
            collecting_confirmed: [false; 5],
            pinging_confirmed_count: 0,
            collecting_confirmed_count: 0,
        }
    }

    fn cluster_view(&self) -> ModelClusterView {
        ModelClusterView {
            peer_state: self.peer_state,
            exit_mode: self.exit_mode,
            is_pinging_completed: self.is_pinging_completed,
            readiness_exited: self.exit_mode.is_some(),
            pinging_confirmed_count: self.pinging_confirmed_count,
            collecting_confirmed_count: self.collecting_confirmed_count,
            required_count: self.required_count,
        }
    }

    fn process(&mut self, command: Command) -> alloc::vec::Vec<Outcome> {
        if self.initial {
            match command {
                Command::ParticipationObserved { .. }
                | Command::ReadyObserved { .. }
                | Command::LocalParticipationCompleted => {
                    self.initial = false;
                    self.peer_state = ModelLifecycleState::Pinging;
                }
                _ => return Vec::new(),
            }
        }

        if self.has_exited() {
            if self.peer_state == ModelLifecycleState::Bootstrapped {
                if let Command::ParticipationObserved { peer_id } = command {
                    return match self.peer_index(peer_id) {
                        Some(_) => vec![Outcome::AcknowledgeRejoin { peer_id }],
                        None => vec![Outcome::NonMemberIgnored { peer_id }],
                    };
                }
            }
            return Vec::new();
        }

        if self.is_pinging_completed {
            match command {
                Command::ParticipationObserved { .. } | Command::LocalParticipationCompleted => {
                    return Vec::new()
                }
                _ => {}
            }
        }

        match command {
            Command::ParticipationObserved { peer_id } => {
                self.apply_participation_observed(peer_id)
            }
            Command::ReadyObserved { peer_id } => self.apply_ready_observed(peer_id),
            Command::LocalParticipationCompleted => self.apply_is_pinging_completedd(),
            Command::DeadlineExpired => self.apply_deadline_expired(),
            Command::Probe => unreachable!("Probe handled in Faction::process"),
        }
    }

    fn apply_participation_observed(&mut self, peer_id: u64) -> alloc::vec::Vec<Outcome> {
        if self.has_exited() {
            return vec![];
        }

        let Some(index) = self.peer_index(peer_id) else {
            return vec![Outcome::NonMemberIgnored { peer_id }];
        };

        if self.pinging_confirmed[index] {
            return vec![Outcome::DuplicateParticipationIgnored { peer_id }];
        }

        self.pinging_confirmed[index] = true;
        self.pinging_confirmed_count += 1;

        vec![Outcome::ParticipationAccepted { peer_id }]
    }

    fn apply_ready_observed(&mut self, peer_id: u64) -> alloc::vec::Vec<Outcome> {
        if self.has_exited() {
            return vec![];
        }

        let Some(index) = self.peer_index(peer_id) else {
            return vec![Outcome::NonMemberIgnored { peer_id }];
        };

        if self.collecting_confirmed[index] {
            return vec![Outcome::DuplicateReadyIgnored { peer_id }];
        }

        self.collecting_confirmed[index] = true;
        self.collecting_confirmed_count += 1;

        if self.is_pinging_completed && self.collecting_confirmed_count >= self.required_count {
            self.exit_mode = Some(Conclusion::Bootstrapped);
            self.peer_state = ModelLifecycleState::Bootstrapped;
            vec![
                Outcome::ReadyAccepted { peer_id },
                Outcome::Concluded {
                    mode: Conclusion::Bootstrapped,
                },
            ]
        } else {
            vec![Outcome::ReadyAccepted { peer_id }]
        }
    }

    fn apply_is_pinging_completedd(&mut self) -> alloc::vec::Vec<Outcome> {
        if self.has_exited() || self.is_pinging_completed {
            return vec![];
        }

        self.is_pinging_completed = true;
        self.peer_state = ModelLifecycleState::Collecting;

        let local_index = self
            .peer_index(self.local_peer_id)
            .expect("local peer must be in peer set");
        if !self.collecting_confirmed[local_index] {
            self.collecting_confirmed[local_index] = true;
            self.collecting_confirmed_count += 1;
        }

        let mut outputs = vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ];

        if self.collecting_confirmed_count >= self.required_count {
            self.exit_mode = Some(Conclusion::Bootstrapped);
            self.peer_state = ModelLifecycleState::Bootstrapped;
            outputs.push(Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            });
        }

        outputs
    }

    fn apply_deadline_expired(&mut self) -> alloc::vec::Vec<Outcome> {
        if self.has_exited() {
            return vec![];
        }

        self.exit_mode = Some(Conclusion::TimedOut);
        self.peer_state = ModelLifecycleState::TimedOut;

        vec![Outcome::Concluded {
            mode: Conclusion::TimedOut,
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
}

fn model_snapshot(cluster_view: ClusterView) -> ModelClusterView {
    ModelClusterView {
        peer_state: match cluster_view.peer_state() {
            PeerState::Fresh => ModelLifecycleState::Fresh,
            PeerState::Pinging => ModelLifecycleState::Pinging,
            PeerState::Collecting => ModelLifecycleState::Collecting,
            PeerState::Bootstrapped => ModelLifecycleState::Bootstrapped,
            PeerState::TimedOut => ModelLifecycleState::TimedOut,
        },
        exit_mode: cluster_view.conclusion(),
        is_pinging_completed: cluster_view.is_pinging_completed(),
        readiness_exited: cluster_view.is_concluded(),
        pinging_confirmed_count: cluster_view.pinging_peers().len(),
        collecting_confirmed_count: cluster_view.collecting_peers().len(),
        required_count: cluster_view.required_count(),
    }
}

fn test_config() -> Config {
    Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4))
}

fn faction() -> Faction {
    Faction::new(test_config(), Box::new(NoOpObserver))
}

fn input_strategy() -> impl Strategy<Value = Command> {
    let participation = (0u64..=6).prop_map(|peer_id| Command::ParticipationObserved { peer_id });
    let ready = (0u64..=6).prop_map(|peer_id| Command::ReadyObserved { peer_id });

    prop_oneof![
        participation,
        ready,
        Just(Command::LocalParticipationCompleted),
        Just(Command::DeadlineExpired),
    ]
}

proptest! {
    #[test]
    fn model_matches_cluster_view_and_outputs_for_random_sequences(
        commands in prop::collection::vec(input_strategy(), 0..64)
    ) {
        // Arrange
        let mut faction = faction();
        let mut model = ModelCoordinator::new();

        // Act
        for command in commands {
            let actual_outputs = match faction.process(command) {
                ProcessResult::Accepted { outcomes, .. } => outcomes,
                ProcessResult::Probed { .. } => unreachable!(),
                ProcessResult::Rejected { .. } => vec![],
            };
            let cluster_view = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };
            let expected_outputs = model.process(command);
            let expected_cluster_view = model.cluster_view();

            // Assert
            prop_assert_eq!(actual_outputs.as_slice(), expected_outputs.as_slice());
            prop_assert_eq!(model_snapshot(cluster_view), expected_cluster_view);
        }
    }
}
