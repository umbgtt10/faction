// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::quorum_policy::QuorumPolicy;
use faction::PeerId;

use faction_protocol::protocol::Protocol;

pub struct ProtocolHarness {
    protocols: Vec<Protocol>,
    peer_ids: Vec<PeerId>,
}

impl ProtocolHarness {
    #[must_use]
    pub fn new(count: usize, required: usize) -> Self {
        let peer_ids: Vec<PeerId> = (0..count as PeerId).collect();

        let protocols = peer_ids
            .iter()
            .map(|&id| {
                let config = Config::new(
                    id,
                    peer_ids.clone(),
                    QuorumPolicy::new(required),
                    FreshnessPolicy::new(2),
                );
                Protocol::new(
                    Faction::new(config, Box::new(NoOpObserver)),
                    peer_ids.clone(),
                    id,
                )
            })
            .collect();

        Self {
            protocols,
            peer_ids,
        }
    }

    #[must_use]
    pub fn peer_ids(&self) -> &[PeerId] {
        &self.peer_ids
    }
}
