// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::conclusion::Conclusion;
use crate::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    ParticipationAccepted { peer_id: PeerId },
    ReadyAccepted { peer_id: PeerId },
    DuplicateParticipationIgnored { peer_id: PeerId },
    DuplicateReadyIgnored { peer_id: PeerId },
    NonMemberIgnored { peer_id: PeerId },
    LocalParticipationCompleted,
    BroadcastLocalReady,
    AcknowledgeRejoin { peer_id: PeerId },
    Concluded { mode: Conclusion },
}
