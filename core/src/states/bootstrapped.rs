// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::node_state::NodeState;
use crate::outcome::Outcome;
use crate::state::State;

pub struct Bootstrapped {
    pub pinging_count: usize,
    pub collecting_count: usize,
}

impl State for Bootstrapped {
    fn step(&self, _input: Command, _config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        unreachable!("accept() rejects all inputs for this state")
    }

    fn accept(&self, _input: &Command) -> bool {
        false
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        previous
            .with_node_state(NodeState::Bootstrapped)
            .with_is_pinging_completed(true)
            .with_pinging_count(self.pinging_count)
            .with_collecting_count(self.collecting_count)
    }

    fn admissible_commands(&self) -> alloc::vec::Vec<Command> {
        vec![Command::Probe]
    }
}
