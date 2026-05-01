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
    outcome: Outcome,
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
            outcome,
            confirmed_peers: new_confirmed_peers,
            confirmed_new,
            quorum_threshold,
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
        match self.quorum_threshold {
            Some(threshold) if self.confirmed_new && self.confirmed_peers.len() >= threshold => {
                vec![
                    self.outcome.clone(),
                    Outcome::ReadyQuorumReached,
                    Outcome::Exited {
                        mode: ExitMode::Bootstrapped,
                    },
                ]
            }
            _ => vec![self.outcome.clone()],
        }
    }
}
