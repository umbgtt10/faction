// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::state::State;
use crate::types::PeerId;

#[derive(Default)]
pub struct Bootstrapped {
    pinged_peers: Vec<PeerId>,
    collected_peers: Vec<PeerId>,
}

impl Bootstrapped {
    #[must_use]
    pub fn new(pinged_peers: Vec<PeerId>, collected_peers: Vec<PeerId>) -> Self {
        Self {
            pinged_peers,
            collected_peers,
        }
    }
}

impl State for Bootstrapped {
    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        let stay = Box::new(Self::new(
            self.pinged_peers.clone(),
            self.collected_peers.clone(),
        ));
        match command {
            Command::ParticipationObserved { peer_id } => {
                let outcome = if config.is_member(peer_id) {
                    Outcome::AcknowledgeRejoin { peer_id }
                } else {
                    Outcome::NonMemberIgnored { peer_id }
                };
                (vec![outcome], stay)
            }
            _ => unreachable!("accept() rejects this command for Bootstrapped"),
        }
    }

    fn accept(&self, command: &Command) -> bool {
        matches!(command, Command::ParticipationObserved { .. })
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        previous
            .clone()
            .with_peer_state(PeerState::Bootstrapped)
            .with_is_pinging_completed(true)
            .with_pinging_peers(self.pinged_peers.clone())
            .with_collecting_peers(self.collected_peers.clone())
    }

    fn admissible_commands(&self) -> Vec<Command> {
        vec![
            Command::ParticipationObserved { peer_id: 0 },
            Command::Probe,
        ]
    }
}
