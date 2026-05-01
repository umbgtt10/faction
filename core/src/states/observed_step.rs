// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec;
use alloc::vec::Vec;

use crate::exit_mode::ExitMode;
use crate::freshness_classification::FreshnessClassification;
use crate::outcome::Outcome;
use crate::PeerId;

pub struct ObservedStep {
    outputs_prefix: Vec<Outcome>,
    confirmed_peers: Vec<PeerId>,
    confirmed_new: bool,
    quorum_threshold: Option<usize>,
}

impl ObservedStep {
    #[must_use]
    pub fn new(
        classification: FreshnessClassification,
        confirmed_peers: Vec<PeerId>,
        peer_id: PeerId,
        kind: ObservedKind,
        quorum_threshold: Option<usize>,
    ) -> Self {
        let is_dup = confirmed_peers.contains(&peer_id);
        let is_stale = matches!(classification, FreshnessClassification::Stale);
        let confirmed_new = !is_dup && !is_stale;

        let outcome = ObservedOutput::new(kind, peer_id).compute_output(classification, is_dup);

        let mut new_confirmed_peers = confirmed_peers;
        if confirmed_new {
            new_confirmed_peers.push(peer_id);
        }

        Self {
            outputs_prefix: vec![outcome],
            confirmed_peers: new_confirmed_peers,
            confirmed_new,
            quorum_threshold,
        }
    }

    #[must_use]
    pub fn new_local(
        confirmed_peers: Vec<PeerId>,
        peer_id: PeerId,
        quorum_threshold: usize,
    ) -> Self {
        let was_present = confirmed_peers.contains(&peer_id);
        let mut new_confirmed_peers = confirmed_peers;
        if !was_present {
            new_confirmed_peers.push(peer_id);
        }

        Self {
            outputs_prefix: vec![
                Outcome::LocalParticipationCompleted,
                Outcome::BroadcastLocalReady,
            ],
            confirmed_peers: new_confirmed_peers,
            confirmed_new: true,
            quorum_threshold: Some(quorum_threshold),
        }
    }

    #[must_use]
    pub fn confirmed_peers(&self) -> Vec<PeerId> {
        self.confirmed_peers.to_vec()
    }

    #[must_use]
    pub fn is_quorum(&self) -> bool {
        matches!(
            self.quorum_threshold,
            Some(threshold) if self.confirmed_new && self.confirmed_peers.len() >= threshold
        )
    }

    #[must_use]
    pub fn outputs(&self) -> Vec<Outcome> {
        let has_quorum = matches!(
            self.quorum_threshold,
            Some(threshold) if self.confirmed_new && self.confirmed_peers.len() >= threshold
        );
        if has_quorum {
            let mut v = self.outputs_prefix.clone();
            v.push(Outcome::ReadyQuorumReached);
            v.push(Outcome::Exited {
                mode: ExitMode::Bootstrapped,
            });
            v
        } else {
            self.outputs_prefix.clone()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedKind {
    Participation,
    Ready,
}

pub struct ObservedOutput {
    kind: ObservedKind,
    peer_id: PeerId,
}

impl ObservedOutput {
    #[must_use]
    pub fn new(kind: ObservedKind, peer_id: PeerId) -> Self {
        Self { kind, peer_id }
    }

    #[must_use]
    pub fn compute_output(&self, classification: FreshnessClassification, is_dup: bool) -> Outcome {
        if matches!(classification, FreshnessClassification::Stale) {
            return self.stale_output();
        }
        if is_dup {
            return self.duplicate_output();
        }
        let timely = matches!(classification, FreshnessClassification::Timely);
        self.accepted_output(timely)
    }

    fn stale_output(&self) -> Outcome {
        match self.kind {
            ObservedKind::Participation => Outcome::StaleParticipationIgnored {
                peer_id: self.peer_id,
            },
            ObservedKind::Ready => Outcome::StaleReadyIgnored {
                peer_id: self.peer_id,
            },
        }
    }

    fn duplicate_output(&self) -> Outcome {
        match self.kind {
            ObservedKind::Participation => Outcome::DuplicateParticipationIgnored {
                peer_id: self.peer_id,
            },
            ObservedKind::Ready => Outcome::DuplicateReadyIgnored {
                peer_id: self.peer_id,
            },
        }
    }

    fn accepted_output(&self, timely: bool) -> Outcome {
        match (self.kind, timely) {
            (ObservedKind::Participation, true) => Outcome::ParticipationAccepted {
                peer_id: self.peer_id,
            },
            (ObservedKind::Participation, false) => Outcome::DelayedParticipationAccepted {
                peer_id: self.peer_id,
            },
            (ObservedKind::Ready, true) => Outcome::ReadyAccepted {
                peer_id: self.peer_id,
            },
            (ObservedKind::Ready, false) => Outcome::DelayedReadyAccepted {
                peer_id: self.peer_id,
            },
        }
    }
}
