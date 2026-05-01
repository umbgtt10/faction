// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::Freshness;
use faction::PeerId;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::observer::Observer;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;

use crate::transport::transport_trait::Transport;

pub struct FactionNode {
    faction: Faction,
    transport: Box<dyn Transport>,
}

impl FactionNode {
    pub fn new(
        peer_id: PeerId,
        peers: Vec<PeerId>,
        quorum_policy: QuorumPolicy,
        freshness_policy: FreshnessPolicy,
        observer: Box<dyn Observer>,
        transport: Box<dyn Transport>,
    ) -> Self {
        let config = Config::new(peer_id, peers, quorum_policy, freshness_policy);
        Self {
            faction: Faction::new(config, observer),
            transport,
        }
    }

    pub fn step(&mut self, command: Command) -> Vec<Outcome> {
        match self.faction.process(command) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => Vec::new(),
        }
    }

    pub fn pump(&mut self, current_marker: Freshness) {
        while let Some((_, command)) = self.transport.recv() {
            let outcomes = self.step(command);

            for outcome in &outcomes {
                if let Outcome::BroadcastLocalReady = outcome {
                    for to in self.faction.config().peers() {
                        if *to != self.faction.config().peer_id() {
                            self.transport.send(
                                *to,
                                Command::ReadyObserved {
                                    peer_id: self.faction.config().peer_id(),
                                    freshness: current_marker,
                                    current_marker,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}
