// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::Arc;
use std::sync::Mutex;

use faction::peer_state::PeerState;

use crate::faction_node::FactionNode;

pub enum Node {
    Task {
        node: Arc<Mutex<FactionNode>>,
    },
    Thread {
        node: Arc<Mutex<FactionNode>>,
        _handle: std::thread::JoinHandle<()>,
    },
    Process {
        child: std::process::Child,
    },
}

impl Node {
    #[must_use]
    pub fn task(node: Arc<Mutex<FactionNode>>) -> Self {
        Self::Task { node }
    }

    #[must_use]
    pub fn thread(node: Arc<Mutex<FactionNode>>, handle: std::thread::JoinHandle<()>) -> Self {
        Self::Thread {
            node,
            _handle: handle,
        }
    }

    #[must_use]
    pub fn process(child: std::process::Child) -> Self {
        Self::Process { child }
    }

    pub fn peer_state(&self) -> PeerState {
        match self {
            Self::Task { node } | Self::Thread { node, .. } => node.lock().unwrap().peer_state(),
            Self::Process { .. } => PeerState::Fresh,
        }
    }
}
