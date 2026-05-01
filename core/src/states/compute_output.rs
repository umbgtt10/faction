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
    kind: ObservedKind,
    peer_id: PeerId,
}

impl ObservedOutput {
    #[must_use]
    pub fn new(kind: ObservedKind, peer_id: PeerId) -> Self {
        Self { kind, peer_id }
    }

    #[must_use]
    pub fn compute_output(
        &self,
        is_member: bool,
        classification: Option<FreshnessClassification>,
        is_dup: bool,
    ) -> Outcome {
        if !is_member {
            return Outcome::NonMemberIgnored {
                peer_id: self.peer_id,
            };
        }
        if matches!(classification, Some(FreshnessClassification::Stale)) {
            return self.stale_output();
        }
        if is_dup {
            return self.duplicate_output();
        }
        let timely = matches!(classification, Some(FreshnessClassification::Timely));
        self.accepted_output(timely)
    }

    #[must_use]
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

    #[must_use]
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

    #[must_use]
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
