// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::{Freshness, PeerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    ParticipationObserved {
        peer_id: PeerId,
        freshness: Freshness,
        current_marker: Freshness,
    },
    ReadyObserved {
        peer_id: PeerId,
        freshness: Freshness,
        current_marker: Freshness,
    },
    LocalParticipationCompleted,
    DeadlineExpired,
    GetSnapshot,
}
