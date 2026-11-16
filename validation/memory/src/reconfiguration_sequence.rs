// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::allocation_counter::AllocationCounter;
use crate::memory_profile::MemoryProfile;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::quorum_policy::QuorumPolicy;

const NODE: u64 = 1;

// A node concluded on its own completion, then membership changes one after
// another: each a newcomer of its own -- added, its membership acknowledged,
// the addition committed -- and then removed, the removal committed, so the
// members are again the configuration's. Each command is built only when it is
// given. The counter runs from before the machine is created.
pub struct ReconfigurationSequence<'a> {
    counter: &'a dyn AllocationCounter,
    peers: u64,
}

impl<'a> ReconfigurationSequence<'a> {
    pub fn new(counter: &'a dyn AllocationCounter, peers: u64) -> Self {
        Self { counter, peers }
    }

    pub fn run(&self, changes: u64) -> MemoryProfile {
        self.counter.start();
        let mut faction = Faction::new(
            Config::new(NODE, (NODE..=self.peers).collect(), QuorumPolicy::new(1), 1),
            Box::new(NoOpObserver),
        );
        let skeleton = self.counter.live();
        let _ = faction.process(Command::LocalParticipationCompleted);
        let peers = self.peers;
        for command in (1..=changes).flat_map(|change| {
            let peer_id = peers + change;
            [
                Command::AdditionRequested { peer_id },
                Command::MembershipAcknowledged { peer_id },
                Command::AdditionCommitted { peer_id },
                Command::RemovalRequested { peer_id },
                Command::RemovalCommitted { peer_id },
            ]
        }) {
            let _ = faction.process(command);
        }
        let held_after_all = self.counter.live();
        drop(faction);
        let after_drop = self.counter.live();
        self.counter.stop();
        MemoryProfile {
            skeleton,
            held_after_all,
            after_drop,
        }
    }
}
