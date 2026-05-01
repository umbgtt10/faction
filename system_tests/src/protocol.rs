// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::faction::Faction;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;

pub struct Protocol {
    faction: Faction,
}

impl Protocol {
    pub fn new(faction: Faction) -> Self {
        Self { faction }
    }

    pub fn evaluate(&mut self, input_message: Command) -> Vec<Outcome> {
        match self.faction.process(input_message) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => Vec::new(),
        }
    }
}
