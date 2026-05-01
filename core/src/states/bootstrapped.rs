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

pub struct Bootstrapped {
    pub pinging_count: usize,
    pub collecting_count: usize,
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
    }

    fn admissible_commands(&self) -> alloc::vec::Vec<Command> {
        vec![Command::Probe]
    }
}
