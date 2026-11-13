// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::io::BufReader;
use std::net::SocketAddr;
use std::process::Child;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread::JoinHandle;
use std::thread::spawn;
use std::time::Duration;

use faction::peer_state::PeerState;
use faction::types::PeerId;

use crate::faction_node::FactionNode;
use crate::node_command::NodeCommand;
use crate::node_snapshot::NodeSnapshot;
use crate::process_node::process_channel::ProcessChannel;
use crate::process_node::state_reader::StateReader;
use crate::thread_runner::ThreadRunner;
use crate::time_ports::TimePorts;

const FRESH: NodeSnapshot = NodeSnapshot {
    peer_state: PeerState::Fresh,
    member_count: 0,
};

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
        channel: ProcessChannel,
        snapshot: Arc<Mutex<NodeSnapshot>>,
        reader: Option<JoinHandle<()>>,
    },
}

impl Node {
    #[must_use]
    pub fn process(mut child: Child) -> Self {
        let stdin = child.stdin.take().expect("child stdin is piped");
        let stdout = child.stdout.take().expect("child stdout is piped");
        let snapshot = Arc::new(Mutex::new(FRESH));
        let (acks, ack_receiver) = channel();
        let reader = StateReader::new(snapshot.clone(), acks).spawn(BufReader::new(stdout));
        Self::Process {
            child: Mutex::new(child),
            channel: ProcessChannel::new(Box::new(stdin), ack_receiver),
            snapshot,
            reader: Some(reader),
        }
    }

    #[must_use]
    pub fn task(node: Rc<RefCell<FactionNode>>) -> Self {
        Self::Task { node }
    }

    #[must_use]
    pub fn spawn_thread(
        build: impl FnOnce() -> FactionNode + Send + 'static,
        time: TimePorts,
        idle_delay: Duration,
    ) -> Self {
        let snapshot = Arc::new(Mutex::new(FRESH));
        let thread_snapshot = snapshot.clone();
        let (commands, command_receiver) = channel();
        let handle = spawn(move || {
            ThreadRunner::new(build(), command_receiver, thread_snapshot, time, idle_delay).run();
        });
        Self::Thread {
            snapshot,
            commands,
            handle: Some(handle),
        }
    }

    pub fn add_peer_address(&self, peer_id: PeerId, addr: SocketAddr) {
        if let Self::Process { channel, .. } = self {
            channel.send(&format!("peer {peer_id} {addr}"));
        }
    }

    pub fn admit(&self, peer_id: PeerId) {
        match self {
            Self::Task { node } => node.borrow_mut().admit(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::Admit(peer_id, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("admit {peer_id}")),
        }
    }

    pub fn deny(&self, peer_id: PeerId) {
        match self {
            Self::Task { node } => node.borrow_mut().deny(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::Deny(peer_id, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("deny {peer_id}")),
        }
    }

    pub fn expire_deadline(&self) {
        match self {
            Self::Task { node } => node.borrow_mut().expire_deadline(),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, NodeCommand::ExpireDeadline);
            }
            Self::Process { channel, .. } => channel.send("expire"),
        }
    }

    #[must_use]
    pub fn member_count(&self) -> usize {
        match self {
            Self::Task { node } => node.borrow_mut().member_count(),
            Self::Thread { snapshot, .. } | Self::Process { snapshot, .. } => {
                snapshot.lock().unwrap().member_count
            }
        }
    }

    #[must_use]
    pub fn peer_state(&self) -> PeerState {
        match self {
            Self::Task { node } => node.borrow_mut().peer_state(),
            Self::Thread { snapshot, .. } | Self::Process { snapshot, .. } => {
                snapshot.lock().unwrap().peer_state
            }
        }
    }

    pub fn request_join(&self, peer_id: PeerId) {
        match self {
            Self::Task { node } => node.borrow_mut().request_join(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::RequestJoin(peer_id, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("request-join {peer_id}")),
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
            Self::Process { child, reader, .. } => {
                if let Ok(child) = child.get_mut() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                if let Some(reader) = reader.take() {
                    let _ = reader.join();
                }
            }
            Self::Task { .. } => {}
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

    fn send_command(commands: &Sender<NodeCommand>, make: impl FnOnce(Sender<()>) -> NodeCommand) {
        let (ack, ack_receiver) = channel::<()>();
        let _ = commands.send(make(ack));
        let _ = ack_receiver.recv();
    }
}
