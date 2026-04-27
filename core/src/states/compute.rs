// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec;
use alloc::vec::Vec;

use crate::freshness_classification::FreshnessClassification;
use crate::vibe_output::VibeOutput;
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
    pub fn compute(
        &self,
        index: Option<usize>,
        classification: Option<FreshnessClassification>,
        is_dup: bool,
    ) -> Vec<VibeOutput> {
        if index.is_none() {
            vec![VibeOutput::NonMemberIgnored {
                peer_id: self.peer_id,
            }]
        } else if matches!(classification, Some(FreshnessClassification::Stale)) {
            vec![self.stale_output()]
        } else if is_dup {
            vec![self.duplicate_output()]
        } else {
            let timely = matches!(classification, Some(FreshnessClassification::Timely));
            vec![self.accepted_output(timely)]
        }
    }

    #[must_use]
    fn stale_output(&self) -> VibeOutput {
        match self.kind {
            ObservedKind::Participation => VibeOutput::StaleParticipationIgnored {
                peer_id: self.peer_id,
            },
            ObservedKind::Ready => VibeOutput::StaleReadyIgnored {
                peer_id: self.peer_id,
            },
        }
    }

    #[must_use]
    fn duplicate_output(&self) -> VibeOutput {
        match self.kind {
            ObservedKind::Participation => VibeOutput::DuplicateParticipationIgnored {
                peer_id: self.peer_id,
            },
            ObservedKind::Ready => VibeOutput::DuplicateReadyIgnored {
                peer_id: self.peer_id,
            },
        }
    }

    #[must_use]
    fn accepted_output(&self, timely: bool) -> VibeOutput {
        match (self.kind, timely) {
            (ObservedKind::Participation, true) => VibeOutput::ParticipationAccepted {
                peer_id: self.peer_id,
            },
            (ObservedKind::Participation, false) => VibeOutput::DelayedParticipationAccepted {
                peer_id: self.peer_id,
            },
            (ObservedKind::Ready, true) => VibeOutput::ReadyAccepted {
                peer_id: self.peer_id,
            },
            (ObservedKind::Ready, false) => VibeOutput::DelayedReadyAccepted {
                peer_id: self.peer_id,
            },
        }
    }
}
