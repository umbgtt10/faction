// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::Arc;
use std::sync::Mutex;

use faction::cluster_view::ClusterView;
use faction::peer_state::PeerState;

pub enum Node {
    Task {
        cluster_view: Arc<Mutex<ClusterView>>,
    },
    Thread {
        cluster_view: Arc<Mutex<ClusterView>>,
        _handle: std::thread::JoinHandle<()>,
    },
    Process {
        child: std::process::Child,
    },
}

impl Node {
    #[must_use]
    pub fn task(cluster_view: Arc<Mutex<ClusterView>>) -> Self {
        Self::Task { cluster_view }
    }

    #[must_use]
    pub fn thread(
        cluster_view: Arc<Mutex<ClusterView>>,
        handle: std::thread::JoinHandle<()>,
    ) -> Self {
        Self::Thread {
            cluster_view,
            _handle: handle,
        }
    }

    #[must_use]
    pub fn process(child: std::process::Child) -> Self {
        Self::Process { child }
    }

    pub fn peer_state(&self) -> PeerState {
        match self {
            Self::Task { cluster_view } | Self::Thread { cluster_view, .. } => {
                cluster_view.lock().unwrap().peer_state()
            }
            Self::Process { .. } => PeerState::Fresh,
        }
    }
}

pub struct Cluster {
    nodes: Vec<Node>,
    required: usize,
}

impl Cluster {
    #[must_use]
    pub fn new(nodes: Vec<Node>, required: usize) -> Self {
        Self { nodes, required }
    }

    #[must_use]
    pub fn is_bootstrapped(&self) -> bool {
        self.nodes
            .iter()
            .all(|node| node.peer_state() == PeerState::Bootstrapped)
    }
}
