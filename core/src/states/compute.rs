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

#[must_use]
fn non_member_output(kind: ObservedKind, peer_id: PeerId) -> VibeOutput {
    match kind {
        ObservedKind::Participation => VibeOutput::NonMemberIgnored { peer_id },
        ObservedKind::Ready => VibeOutput::NonMemberIgnored { peer_id },
    }
}

#[must_use]
fn stale_output(kind: ObservedKind, peer_id: PeerId) -> VibeOutput {
    match kind {
        ObservedKind::Participation => VibeOutput::StaleParticipationIgnored { peer_id },
        ObservedKind::Ready => VibeOutput::StaleReadyIgnored { peer_id },
    }
}

#[must_use]
fn duplicate_output(kind: ObservedKind, peer_id: PeerId) -> VibeOutput {
    match kind {
        ObservedKind::Participation => VibeOutput::DuplicateParticipationIgnored { peer_id },
        ObservedKind::Ready => VibeOutput::DuplicateReadyIgnored { peer_id },
    }
}

#[must_use]
fn accepted_output(kind: ObservedKind, peer_id: PeerId, timely: bool) -> VibeOutput {
    match (kind, timely) {
        (ObservedKind::Participation, true) => VibeOutput::ParticipationAccepted { peer_id },
        (ObservedKind::Participation, false) => {
            VibeOutput::DelayedParticipationAccepted { peer_id }
        }
        (ObservedKind::Ready, true) => VibeOutput::ReadyAccepted { peer_id },
        (ObservedKind::Ready, false) => VibeOutput::DelayedReadyAccepted { peer_id },
    }
}

#[must_use]
pub fn observed_output(
    kind: ObservedKind,
    peer_id: PeerId,
    index: Option<usize>,
    classification: Option<FreshnessClassification>,
    is_dup: bool,
) -> Vec<VibeOutput> {
    if index.is_none() {
        vec![non_member_output(kind, peer_id)]
    } else if matches!(classification, Some(FreshnessClassification::Stale)) {
        vec![stale_output(kind, peer_id)]
    } else if is_dup {
        vec![duplicate_output(kind, peer_id)]
    } else {
        let timely = matches!(classification, Some(FreshnessClassification::Timely));
        vec![accepted_output(kind, peer_id, timely)]
    }
}
