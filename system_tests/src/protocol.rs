// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::faction::Faction;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;

#[derive(Debug, Clone)]
pub enum Decision {
    BroadcastReady,
    Noop,
}

pub struct Protocol {
    faction: Faction,
}

impl Protocol {
    pub fn new(faction: Faction) -> Self {
        Self { faction }
    }

    pub fn decide(&mut self, input_message: Command) -> Decision {
        let outcomes = match self.faction.process(input_message) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => return Decision::Noop,
        };

        for outcome in outcomes {
            if let Outcome::BroadcastLocalReady = outcome {
                return Decision::BroadcastReady;
            }
        }

        Decision::Noop
    }
}
