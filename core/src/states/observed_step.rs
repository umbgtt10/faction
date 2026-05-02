// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec;
use alloc::vec::Vec;

use crate::exit_mode::ExitMode;
use crate::freshness_classification::FreshnessClassification;
use crate::outcome::Outcome;
use crate::PeerId;

use super::compute_output::ObservedKind;
use super::compute_output::ObservedOutput;

pub struct ObservedStep {
    outcomes: Vec<Outcome>,
    confirmed_peers: Vec<PeerId>,
    is_quorum: bool,
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

        let mut new_confirmed_peers = confirmed_peers;
        if confirmed_new {
            new_confirmed_peers.push(peer_id);
        }

        let is_quorum =
            quorum_threshold.is_some_and(|t| confirmed_new && new_confirmed_peers.len() >= t);

        let mut outcomes = vec![ObservedOutput::new(kind, peer_id, classification, is_dup)
            .outcome()
            .clone()];
        if is_quorum {
            outcomes.push(Outcome::Exited {
                mode: ExitMode::Bootstrapped,
            });
        }

        Self {
            outcomes,
            confirmed_peers: new_confirmed_peers,
            is_quorum,
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

        let is_quorum = new_confirmed_peers.len() >= quorum_threshold;
        let mut outcomes = vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ];

        if is_quorum {
            outcomes.push(Outcome::Exited {
                mode: ExitMode::Bootstrapped,
            });
        }

        Self {
            outcomes,
            confirmed_peers: new_confirmed_peers,
            is_quorum,
        }
    }

    #[must_use]
    pub fn confirmed_peers(&self) -> &[PeerId] {
        &self.confirmed_peers
    }

    #[must_use]
    pub fn is_quorum(&self) -> bool {
        self.is_quorum
    }

    #[must_use]
    pub fn outcomes(&self) -> &[Outcome] {
        &self.outcomes
    }
}
