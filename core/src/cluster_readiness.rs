// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_readiness_config::ClusterReadinessConfig;
use crate::cluster_readiness_input::ClusterReadinessInput;
use crate::cluster_readiness_observer::ClusterReadinessObserver;
use crate::cluster_readiness_output::ClusterReadinessOutput;
use crate::cluster_readiness_snapshot::ClusterReadinessSnapshot;
use crate::cluster_readiness_transition::ClusterReadinessTransition;
use crate::freshness_classification::FreshnessClassification;
use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::Freshness;
use crate::PeerId;

pub struct ClusterReadiness {
    config: ClusterReadinessConfig,
    observer: Box<dyn ClusterReadinessObserver>,
    lifecycle_state: ReadinessLifecycleState,
    exit_mode: Option<ReadinessExitMode>,
    local_participation_complete: bool,
    phase1_confirmed: Vec<bool>,
    phase2_confirmed: Vec<bool>,
    phase1_confirmed_count: usize,
    phase2_confirmed_count: usize,
}

impl ClusterReadiness {
    #[must_use]
    pub fn new(
        config: ClusterReadinessConfig,
        observer: Box<dyn ClusterReadinessObserver>,
    ) -> Self {
        let peer_count = config.peer_count();

        Self {
            config,
            observer,
            lifecycle_state: ReadinessLifecycleState::Phase1Active,
            exit_mode: None,
            local_participation_complete: false,
            phase1_confirmed: vec![false; peer_count],
            phase2_confirmed: vec![false; peer_count],
            phase1_confirmed_count: 0,
            phase2_confirmed_count: 0,
        }
    }

    #[must_use]
    pub fn apply(&mut self, input: ClusterReadinessInput) -> Vec<ClusterReadinessOutput> {
        let previous_state = self.snapshot();
        let outputs = self.apply_without_observer(input);
        let new_state = self.snapshot();
        let transition =
            ClusterReadinessTransition::new(previous_state, outputs.clone(), new_state);

        self.observer.observe(input, transition);

        outputs
    }

    #[must_use]
    pub fn snapshot(&self) -> ClusterReadinessSnapshot {
        ClusterReadinessSnapshot::new(
            self.lifecycle_state,
            self.exit_mode,
            self.local_participation_complete,
            self.exit_mode.is_some(),
            self.phase1_confirmed_count,
            self.phase2_confirmed_count,
            self.config.quorum_threshold(),
        )
    }

    #[must_use]
    pub fn config(&self) -> &ClusterReadinessConfig {
        &self.config
    }

    fn apply_without_observer(
        &mut self,
        input: ClusterReadinessInput,
    ) -> Vec<ClusterReadinessOutput> {
        match input {
            ClusterReadinessInput::ParticipationObserved {
                peer_id,
                freshness,
                current_marker,
            } => self.apply_participation_observed(peer_id, freshness, current_marker),
            ClusterReadinessInput::ReadyObserved {
                peer_id,
                freshness,
                current_marker,
            } => self.apply_ready_observed(peer_id, freshness, current_marker),
            ClusterReadinessInput::LocalParticipationCompleted => {
                self.apply_local_participation_completed()
            }
            ClusterReadinessInput::DeadlineExpired => self.apply_deadline_expired(),
        }
    }

    fn apply_participation_observed(
        &mut self,
        peer_id: PeerId,
        freshness: Freshness,
        current_marker: Freshness,
    ) -> Vec<ClusterReadinessOutput> {
        if self.has_exited() {
            return vec![ClusterReadinessOutput::StaleParticipationIgnored { peer_id }];
        }

        if !self.config.is_member(peer_id) {
            return vec![ClusterReadinessOutput::NonMemberIgnored { peer_id }];
        }

        let classification = self
            .config
            .freshness_policy()
            .classify(current_marker, freshness);

        if classification == FreshnessClassification::Stale {
            return vec![ClusterReadinessOutput::StaleParticipationIgnored { peer_id }];
        }

        let Some(index) = self.config.peer_index(peer_id) else {
            return vec![ClusterReadinessOutput::NonMemberIgnored { peer_id }];
        };

        if self.phase1_confirmed[index] {
            return vec![ClusterReadinessOutput::DuplicateParticipationIgnored { peer_id }];
        }

        self.phase1_confirmed[index] = true;
        self.phase1_confirmed_count += 1;

        match classification {
            FreshnessClassification::Timely => {
                vec![ClusterReadinessOutput::ParticipationAccepted { peer_id }]
            }
            FreshnessClassification::DelayedWithinMargin => {
                vec![ClusterReadinessOutput::DelayedParticipationAccepted { peer_id }]
            }
            FreshnessClassification::Stale => Vec::new(),
        }
    }

    fn apply_ready_observed(
        &mut self,
        peer_id: PeerId,
        freshness: Freshness,
        current_marker: Freshness,
    ) -> Vec<ClusterReadinessOutput> {
        if self.has_exited() {
            return vec![ClusterReadinessOutput::StaleReadyIgnored { peer_id }];
        }

        if !self.config.is_member(peer_id) {
            return vec![ClusterReadinessOutput::NonMemberIgnored { peer_id }];
        }

        let classification = self
            .config
            .freshness_policy()
            .classify(current_marker, freshness);

        if classification == FreshnessClassification::Stale {
            return vec![ClusterReadinessOutput::StaleReadyIgnored { peer_id }];
        }

        let Some(index) = self.config.peer_index(peer_id) else {
            return vec![ClusterReadinessOutput::NonMemberIgnored { peer_id }];
        };

        if self.phase2_confirmed[index] {
            return vec![ClusterReadinessOutput::DuplicateReadyIgnored { peer_id }];
        }

        self.phase2_confirmed[index] = true;
        self.phase2_confirmed_count += 1;

        let accepted_output = match classification {
            FreshnessClassification::Timely => ClusterReadinessOutput::ReadyAccepted { peer_id },
            FreshnessClassification::DelayedWithinMargin => {
                ClusterReadinessOutput::DelayedReadyAccepted { peer_id }
            }
            FreshnessClassification::Stale => {
                return Vec::new();
            }
        };

        if self.local_participation_complete
            && self.phase2_confirmed_count >= self.config.quorum_threshold()
        {
            self.exit_by_quorum_with_prefix(accepted_output)
        } else {
            vec![accepted_output]
        }
    }

    fn apply_local_participation_completed(&mut self) -> Vec<ClusterReadinessOutput> {
        if self.has_exited() || self.local_participation_complete {
            return Vec::new();
        }

        self.local_participation_complete = true;
        self.lifecycle_state = ReadinessLifecycleState::Phase2Active;

        if let Some(index) = self.config.peer_index(self.config.local_peer_id()) {
            if !self.phase2_confirmed[index] {
                self.phase2_confirmed[index] = true;
                self.phase2_confirmed_count += 1;
            }
        }

        let outputs = vec![
            ClusterReadinessOutput::LocalParticipationCompleted,
            ClusterReadinessOutput::BroadcastLocalReady,
        ];

        if self.phase2_confirmed_count >= self.config.quorum_threshold() {
            self.exit_by_quorum_with_batch(outputs)
        } else {
            outputs
        }
    }

    fn apply_deadline_expired(&mut self) -> Vec<ClusterReadinessOutput> {
        if self.has_exited() {
            return Vec::new();
        }

        self.exit_mode = Some(ReadinessExitMode::Deadline);
        self.lifecycle_state = ReadinessLifecycleState::ReadyByDeadline;

        vec![ClusterReadinessOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline,
        }]
    }

    fn exit_by_quorum_with_prefix(
        &mut self,
        first: ClusterReadinessOutput,
    ) -> Vec<ClusterReadinessOutput> {
        self.exit_mode = Some(ReadinessExitMode::Quorum);
        self.lifecycle_state = ReadinessLifecycleState::ReadyByQuorum;

        vec![
            first,
            ClusterReadinessOutput::ReadyQuorumReached,
            ClusterReadinessOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    }

    fn exit_by_quorum_with_batch(
        &mut self,
        outputs: Vec<ClusterReadinessOutput>,
    ) -> Vec<ClusterReadinessOutput> {
        self.exit_mode = Some(ReadinessExitMode::Quorum);
        self.lifecycle_state = ReadinessLifecycleState::ReadyByQuorum;

        let mut emitted_outputs = outputs;
        emitted_outputs.push(ClusterReadinessOutput::ReadyQuorumReached);
        emitted_outputs.push(ClusterReadinessOutput::ReadinessExited {
            mode: ReadinessExitMode::Quorum,
        });

        emitted_outputs
    }

    const fn has_exited(&self) -> bool {
        self.exit_mode.is_some()
    }
}
