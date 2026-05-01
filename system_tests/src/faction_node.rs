// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::Freshness;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;

use crate::transport::transport_trait::Transport;

pub struct FactionNode {
    faction: Faction,
}

impl FactionNode {
    pub fn new(config: Config) -> Self {
        Self {
            faction: Faction::new(config, Box::new(NoOpObserver)),
        }
    }

    pub fn step(&mut self, command: Command) -> Vec<Outcome> {
        match self.faction.process(command) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => Vec::new(),
        }
    }

    pub fn pump(&mut self, transport: &mut dyn Transport, current_marker: Freshness) {
        while let Some((_, command)) = transport.recv() {
            let outcomes = self.step(command);

            for outcome in &outcomes {
                if let Outcome::BroadcastLocalReady = outcome {
                    for to in self.faction.config().peers() {
                        if *to != self.faction.config().peer_id() {
                            transport.send(
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
