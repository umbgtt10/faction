// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
    pub fn new(kind: ObservedKind, peer_id: PeerId, is_dup: bool) -> Self {
        let outcome = if is_dup {
            Self::duplicate_output(kind, peer_id)
        } else {
            Self::accepted_output(kind, peer_id)
        };
        Self { outcome }
    }

    #[must_use]
    pub fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    fn duplicate_output(kind: ObservedKind, peer_id: PeerId) -> Outcome {
        match kind {
            ObservedKind::Participation => Outcome::DuplicateParticipationIgnored { peer_id },
            ObservedKind::Ready => Outcome::DuplicateReadyIgnored { peer_id },
        }
    }

    fn accepted_output(kind: ObservedKind, peer_id: PeerId) -> Outcome {
        match kind {
            ObservedKind::Participation => Outcome::ParticipationAccepted { peer_id },
            ObservedKind::Ready => Outcome::ReadyAccepted { peer_id },
        }
    }
}
