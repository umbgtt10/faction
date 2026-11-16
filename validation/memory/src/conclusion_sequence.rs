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
use std::iter::repeat_n;

const NODE: u64 = 1;

// A node driven from new to concluded: every other peer's participation, its
// own, then every other peer's readiness, the required count all of them --
// and, when busy, each signal given twice and followed by a probe, a duplicate
// and a query. Each command is built only when it is given, so what the
// counter shows is what the machine holds. The counter runs from before the
// machine is created.
pub struct ConclusionSequence<'a> {
    counter: &'a dyn AllocationCounter,
    peers: u64,
    busy: bool,
}

impl<'a> ConclusionSequence<'a> {
    pub fn new(counter: &'a dyn AllocationCounter, peers: u64, busy: bool) -> Self {
        Self {
            counter,
            peers,
            busy,
        }
    }

    pub fn run(&self) -> MemoryProfile {
        self.counter.start();
        let mut faction = Faction::new(
            Config::new(
                NODE,
                (NODE..=self.peers).collect(),
                QuorumPolicy::new(self.peers as usize),
                1,
            ),
            Box::new(NoOpObserver),
        );
        let skeleton = self.counter.live();
        let given = if self.busy { 2 } else { 1 };
        let busy = self.busy;
        let signals = (NODE + 1..=self.peers)
            .map(|peer_id| Command::ParticipationObserved { peer_id })
            .chain(once(Command::LocalParticipationCompleted))
            .chain((NODE + 1..=self.peers).map(|peer_id| Command::ReadyObserved { peer_id }));
        for command in
            signals.flat_map(|signal| repeat_n(signal, given).chain(busy.then_some(Command::Probe)))
        {
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
