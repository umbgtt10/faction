// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::PeerId;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::quorum_policy::QuorumPolicy;

use faction_protocol::protocol::Protocol;

use rstest::rstest;

const PEER_SET: [PeerId; 5] = [0, 1, 2, 3, 4];
const REQUIRED: usize = 4;

#[rstest]
#[case::in_memory("in-memory")]
fn cluster_reaches_bootstrapped(#[case] _variant: &str) {
    let config = Config::new(
        0,
        PEER_SET.to_vec(),
        QuorumPolicy::new(REQUIRED),
        FreshnessPolicy::new(2),
    );
    let _protocol = Protocol::new(
        Faction::new(config, Box::new(NoOpObserver)),
        PEER_SET.to_vec(),
        0,
    );
}
