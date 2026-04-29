// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::machine_snapshot::MachineSnapshot;
use crate::readiness_exit_mode::ReadinessExitMode;
use crate::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachineOutput {
    ParticipationAccepted { peer_id: PeerId },
    ReadyAccepted { peer_id: PeerId },
    DelayedParticipationAccepted { peer_id: PeerId },
    DelayedReadyAccepted { peer_id: PeerId },
    DuplicateParticipationIgnored { peer_id: PeerId },
    DuplicateReadyIgnored { peer_id: PeerId },
    StaleParticipationIgnored { peer_id: PeerId },
    StaleReadyIgnored { peer_id: PeerId },
    NonMemberIgnored { peer_id: PeerId },
    LocalParticipationCompleted,
    BroadcastLocalReady,
    ReadyQuorumReached,
    ReadinessExited { mode: ReadinessExitMode },
    SnapshotAvailable(MachineSnapshot),
}
