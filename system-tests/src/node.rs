// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::process::Child;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{Sender, channel};
use std::thread::{JoinHandle, spawn};

use faction::peer_state::PeerState;
use faction::types::PeerId;

use crate::faction_node::{FactionNode, NodeCommand, NodeSnapshot};

pub enum Node {
    Task {
        node: Rc<RefCell<FactionNode>>,
    },
    Thread {
        snapshot: Arc<Mutex<NodeSnapshot>>,
        commands: Sender<NodeCommand>,
        handle: Option<JoinHandle<()>>,
    },
    Process {
        child: Mutex<Child>,
    },
}

impl Node {
    #[must_use]
    pub fn task(node: Rc<RefCell<FactionNode>>) -> Self {
        Self::Task { node }
    }

    #[must_use]
    pub fn spawn_thread(build: impl FnOnce() -> FactionNode + Send + 'static) -> Self {
        let snapshot = Arc::new(Mutex::new(NodeSnapshot::fresh()));
        let snapshot_clone = snapshot.clone();
        let (commands, commands_rx) = channel();
        let handle = spawn(move || {
            let mut node = build();
            node.run_until_shutdown(commands_rx, snapshot_clone);
        });
        Self::Thread {
            snapshot,
            commands,
            handle: Some(handle),
        }
    }

    #[must_use]
    pub fn process(child: Child) -> Self {
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

    pub fn request_join(&self, peer_id: PeerId) {
        match self {
            Self::Task { node } => node.borrow_mut().request_join(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::RequestJoin(peer_id, ack));
            }
            Self::Process { .. } => {}
        }
    }

    pub fn admit(&self, peer_id: PeerId) {
        match self {
            Self::Task { node } => node.borrow_mut().admit(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::Admit(peer_id, ack));
            }
            Self::Process { .. } => {}
        }
    }

    pub fn deny(&self, peer_id: PeerId) {
        match self {
            Self::Task { node } => node.borrow_mut().deny(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::Deny(peer_id, ack));
            }
            Self::Process { .. } => {}
        }
    }

    pub fn expire_deadline(&self) {
        match self {
            Self::Task { node } => node.borrow_mut().expire_deadline(),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, NodeCommand::ExpireDeadline);
            }
            Self::Process { .. } => {}
        }
    }

    pub fn member_count(&self) -> usize {
        match self {
            Self::Task { node } => node.borrow_mut().member_count(),
            Self::Thread { snapshot, .. } => snapshot.lock().unwrap().member_count,
            Self::Process { .. } => 0,
        }
    }

    pub fn peer_state(&self) -> PeerState {
        match self {
            Self::Task { node } => node.borrow_mut().peer_state(),
            Self::Thread { snapshot, .. } => snapshot.lock().unwrap().peer_state,
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
        match self {
            Self::Thread {
                commands, handle, ..
            } => {
                let _ = commands.send(NodeCommand::Shutdown);
                if let Some(handle) = handle.take() {
                    let _ = handle.join();
                }
            }
            Self::Process { child } => {
                let c = child.get_mut().unwrap();
                let _ = c.kill();
                let _ = c.wait();
            }
            Self::Task { .. } => {}
        }
    }

    fn send_command(commands: &Sender<NodeCommand>, make: impl FnOnce(Sender<()>) -> NodeCommand) {
        let (ack, ack_rx) = channel::<()>();
        let _ = commands.send(make(ack));
        let _ = ack_rx.recv();
    }
}
