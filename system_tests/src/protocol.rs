// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::PeerId;
use faction::command::Command;
use faction::faction::Faction;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;

use crate::timer::TimerEvent;

#[derive(Debug, Clone)]
pub enum Decision {
    BroadcastReady,
    Schedule(TimerEvent),
    Cancel(TimerEvent),
    Noop,
}

pub struct Protocol {
    faction: Faction,
}

impl Protocol {
    pub fn new(faction: Faction) -> Self {
        Self { faction }
    }

    pub fn start_decisions(&self, peers: &[PeerId], local_peer_id: PeerId) -> Vec<Decision> {
        let mut decisions = Vec::new();

        for peer in peers {
            if *peer != local_peer_id {
                decisions.push(Decision::Schedule(TimerEvent::Fire(
                    Command::ParticipationObserved {
                        peer_id: *peer,
                        freshness: 0,
                        current_marker: 0,
                    },
                )));
            }
        }

        decisions.push(Decision::Schedule(TimerEvent::Fire(
            Command::LocalParticipationCompleted,
        )));

        decisions
    }

    pub fn decide(&mut self, input_message: Command) -> Decision {
        let outcomes = match self.faction.process(input_message) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => return Decision::Noop,
        };

        for outcome in outcomes {
            match outcome {
                Outcome::BroadcastLocalReady => return Decision::BroadcastReady,
                Outcome::Exited { .. } => {
                    return Decision::Cancel(TimerEvent::Fire(
                        Command::LocalParticipationCompleted,
                    ));
                }
                _ => {}
            }
        }

        Decision::Noop
    }
}
