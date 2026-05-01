// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::Arc;
use std::sync::Mutex;

use faction::peer_state::PeerState;
use faction_protocol::protocol::Protocol;

pub enum Node {
    Task {
        protocol: Arc<Mutex<Protocol>>,
    },
    Thread {
        protocol: Arc<Mutex<Protocol>>,
        _handle: std::thread::JoinHandle<()>,
    },
    Process {
        child: std::process::Child,
    },
}

impl Node {
    #[must_use]
    pub fn task(protocol: Arc<Mutex<Protocol>>) -> Self {
        Self::Task { protocol }
    }

    #[must_use]
    pub fn thread(protocol: Arc<Mutex<Protocol>>, handle: std::thread::JoinHandle<()>) -> Self {
        Self::Thread {
            protocol,
            _handle: handle,
        }
    }

    #[must_use]
    pub fn process(child: std::process::Child) -> Self {
        Self::Process { child }
    }

    pub fn peer_state(&self) -> PeerState {
        match self {
            Self::Task { protocol } | Self::Thread { protocol, .. } => {
                protocol.lock().unwrap().cluster_view().peer_state()
            }
            Self::Process { .. } => PeerState::Fresh,
        }
    }
}

pub struct Cluster {
    nodes: Vec<Node>,
}

impl Cluster {
    #[must_use]
    pub fn new(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }

    #[must_use]
    pub fn is_bootstrapped(&self) -> bool {
        self.nodes
            .iter()
            .all(|node| node.peer_state() == PeerState::Bootstrapped)
    }
}
