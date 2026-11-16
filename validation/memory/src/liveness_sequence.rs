// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::allocation_counter::AllocationCounter;
use crate::memory_profile::MemoryProfile;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::quorum_policy::QuorumPolicy;
use std::iter::once;

const MISSED: u64 = 2;
const NODE: u64 = 1;

// A node concluded on its own completion, then probe periods one after
// another: in each, a probe is due, every other member is heard, and one of
// them misses its probe and its indirect probes -- suspected, at a threshold
// of one -- and is heard again, revived. Each command is built only when it is
// given. The counter runs from before the machine is created.
pub struct LivenessSequence<'a> {
    counter: &'a dyn AllocationCounter,
    peers: u64,
}

impl<'a> LivenessSequence<'a> {
    pub fn new(counter: &'a dyn AllocationCounter, peers: u64) -> Self {
        Self { counter, peers }
    }

    pub fn run(&self, periods: u64) -> MemoryProfile {
        self.counter.start();
        let mut faction = Faction::new(
            Config::new(
                NODE,
                (NODE..=self.peers).collect(),
                QuorumPolicy::new(1),
                1,
                0,
            ),
            Box::new(NoOpObserver),
        );
        let skeleton = self.counter.live();
        let _ = faction.process(Command::LocalParticipationCompleted);
        let peers = self.peers;
        for command in (0..periods).flat_map(|_| {
            once(Command::ProbeDue)
                .chain((NODE + 1..=peers).map(|peer_id| Command::PeerHeard { peer_id }))
                .chain([
                    Command::ProbeMissed { peer_id: MISSED },
                    Command::IndirectProbesMissed { peer_id: MISSED },
                    Command::PeerHeard { peer_id: MISSED },
                ])
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
