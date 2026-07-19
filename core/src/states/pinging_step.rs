// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec;
use alloc::vec::Vec;

use crate::outcome::Outcome;
use crate::types::PeerId;

pub struct PingingStep {
    outcomes: Vec<Outcome>,
    confirmed_peers: Vec<PeerId>,
}

impl PingingStep {
    #[must_use]
    pub fn new(confirmed_peers: Vec<PeerId>, peer_id: PeerId) -> Self {
        let is_dup = confirmed_peers.contains(&peer_id);
        let mut new_confirmed_peers = confirmed_peers;
        if !is_dup {
            new_confirmed_peers.push(peer_id);
        }

        let outcome = if is_dup {
            Outcome::DuplicateParticipationIgnored { peer_id }
        } else {
            Outcome::ParticipationAccepted { peer_id }
        };

        Self {
            outcomes: vec![outcome],
            confirmed_peers: new_confirmed_peers,
        }
    }

    #[must_use]
    pub fn confirmed_peers(&self) -> &[PeerId] {
        &self.confirmed_peers
    }

    #[must_use]
    pub fn outcomes(&self) -> &[Outcome] {
        &self.outcomes
    }
}
