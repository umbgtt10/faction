// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use faction::peer_state::PeerState;

use crate::faction_node::FactionNode;

pub enum Node {
    Task {
        node: Rc<RefCell<FactionNode>>,
    },
    Thread {
        state: Arc<Mutex<PeerState>>,
        _handle: std::thread::JoinHandle<()>,
    },
    Process {
        child: Mutex<std::process::Child>,
    },
}

impl Node {
    #[must_use]
    pub fn task(node: Rc<RefCell<FactionNode>>) -> Self {
        Self::Task { node }
    }

    #[must_use]
    pub fn spawn_thread(build: impl FnOnce() -> FactionNode + Send + 'static) -> Self {
        let state = Arc::new(Mutex::new(PeerState::Fresh));
        let state_clone = state.clone();
        let handle = std::thread::spawn(move || {
            let mut node = build();
            node.run();
            *state_clone.lock().unwrap() = node.peer_state();
        });
        Self::Thread {
            state,
            _handle: handle,
        }
    }

    #[must_use]
    pub fn process(child: std::process::Child) -> Self {
        Self::Process {
            child: Mutex::new(child),
        }
    }

    pub fn start(&self) {
        if let Self::Task { node } = self {
            node.borrow_mut().start();
        }
    }

    pub fn step(&self) {
        if let Self::Task { node } = self {
            node.borrow_mut().step();
        }
    }

    pub fn peer_state(&self) -> PeerState {
        match self {
            Self::Task { node } => node.borrow_mut().peer_state(),
            Self::Thread { state, .. } => *state.lock().unwrap(),
            Self::Process { child } => match child.lock().unwrap().try_wait() {
                Ok(Some(status)) if status.success() => PeerState::Bootstrapped,
                Ok(Some(_)) => PeerState::TimedOut,
                Ok(None) => PeerState::Fresh,
                Err(_) => PeerState::TimedOut,
            },
        }
    }

    pub fn wait(&self) {
        if let Self::Process { child } = self {
            let _ = child.lock().unwrap().wait();
        }
    }

    pub fn shutdown(&mut self) {
        if let Self::Process { child } = self {
            let c = child.get_mut().unwrap();
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}
