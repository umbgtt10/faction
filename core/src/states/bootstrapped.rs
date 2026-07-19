// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
    fn step(&self, _command: Command, _config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        unreachable!("accept() rejects all commands for this state")
    }

    fn accept(&self, _command: &Command) -> bool {
        false
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
        vec![Command::Probe]
    }
}
