// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::peer_state::PeerState;
use crate::outcome::Outcome;
use crate::state::State;

pub struct TimedOut {
    pub pinging_count: usize,
    pub collecting_count: usize,
}

impl State for TimedOut {
    fn step(&self, _input: Command, _config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        unreachable!("accept() rejects all inputs for this state")
    }

    fn cluster_view(&self, previous: &ClusterView, _config: &Config) -> ClusterView {
        previous.clone().with_peer_state(PeerState::TimedOut)
    }

    fn accept(&self, _input: &Command) -> bool {
        false
    }

    fn admissible_commands(&self) -> Vec<Command> {
        vec![Command::Probe]
    }
}
