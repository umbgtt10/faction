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
use crate::PeerId;

#[derive(Default)]
pub struct TimedOut {
    pub pinging_peers: Vec<PeerId>,
    pub collecting_peers: Vec<PeerId>,
}

impl TimedOut {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl State for TimedOut {
    fn step(&self, _command: Command, _config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        unreachable!("accept() rejects all commands for this state")
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        previous
            .clone()
            .with_peer_state(PeerState::TimedOut)
            .with_pinging_peers(self.pinging_peers.clone())
            .with_collecting_peers(self.collecting_peers.clone())
    }

    fn accept(&self, _command: &Command) -> bool {
        false
    }

    fn admissible_commands(&self) -> Vec<Command> {
        vec![Command::Probe]
    }
}
