// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::freshness_classification::FreshnessClassification;
use crate::outcome::Outcome;
use crate::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedKind {
    Participation,
    Ready,
}

pub struct ObservedOutput {
    outcome: Outcome,
}

impl ObservedOutput {
    #[must_use]
    pub fn new(
        kind: ObservedKind,
        peer_id: PeerId,
        classification: FreshnessClassification,
        is_dup: bool,
    ) -> Self {
        let outcome = if matches!(classification, FreshnessClassification::Stale) {
            Self::stale_output(kind, peer_id)
        } else if is_dup {
            Self::duplicate_output(kind, peer_id)
        } else {
            let timely = matches!(classification, FreshnessClassification::Timely);
            Self::accepted_output(kind, peer_id, timely)
        };
        Self { outcome }
    }

    #[must_use]
    pub fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    #[must_use]
    pub fn into_outcome(self) -> Outcome {
        self.outcome
    }

    fn stale_output(kind: ObservedKind, peer_id: PeerId) -> Outcome {
        match kind {
            ObservedKind::Participation => Outcome::StaleParticipationIgnored { peer_id },
            ObservedKind::Ready => Outcome::StaleReadyIgnored { peer_id },
        }
    }

    fn duplicate_output(kind: ObservedKind, peer_id: PeerId) -> Outcome {
        match kind {
            ObservedKind::Participation => Outcome::DuplicateParticipationIgnored { peer_id },
            ObservedKind::Ready => Outcome::DuplicateReadyIgnored { peer_id },
        }
    }

    fn accepted_output(kind: ObservedKind, peer_id: PeerId, timely: bool) -> Outcome {
        match (kind, timely) {
            (ObservedKind::Participation, true) => Outcome::ParticipationAccepted { peer_id },
            (ObservedKind::Participation, false) => {
                Outcome::DelayedParticipationAccepted { peer_id }
            }
            (ObservedKind::Ready, true) => Outcome::ReadyAccepted { peer_id },
            (ObservedKind::Ready, false) => Outcome::DelayedReadyAccepted { peer_id },
        }
    }
}
