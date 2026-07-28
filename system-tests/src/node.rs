// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::io::{BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::process::{Child, ChildStdin, ChildStdout};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
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
        stdin: Mutex<ChildStdin>,
        snapshot: Arc<Mutex<NodeSnapshot>>,
        acks: Mutex<Receiver<()>>,
        reader: Option<JoinHandle<()>>,
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
    pub fn process(mut child: Child) -> Self {
        let stdin = child.stdin.take().expect("child stdin is piped");
        let stdout = child.stdout.take().expect("child stdout is piped");
        let snapshot = Arc::new(Mutex::new(NodeSnapshot::fresh()));
        let (ack_tx, ack_rx) = channel();
        let reader = spawn_stdout_reader(stdout, snapshot.clone(), ack_tx);
        Self::Process {
            child: Mutex::new(child),
            stdin: Mutex::new(stdin),
            snapshot,
            acks: Mutex::new(ack_rx),
            reader: Some(reader),
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
            Self::Process { stdin, acks, .. } => {
                send_process_command(stdin, acks, &format!("request-join {peer_id}"));
            }
        }
    }

    pub fn admit(&self, peer_id: PeerId) {
        match self {
            Self::Task { node } => node.borrow_mut().admit(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::Admit(peer_id, ack));
            }
            Self::Process { stdin, acks, .. } => {
                send_process_command(stdin, acks, &format!("admit {peer_id}"));
            }
        }
    }

    pub fn deny(&self, peer_id: PeerId) {
        match self {
            Self::Task { node } => node.borrow_mut().deny(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::Deny(peer_id, ack));
            }
            Self::Process { stdin, acks, .. } => {
                send_process_command(stdin, acks, &format!("deny {peer_id}"));
            }
        }
    }

    pub fn expire_deadline(&self) {
        match self {
            Self::Task { node } => node.borrow_mut().expire_deadline(),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, NodeCommand::ExpireDeadline);
            }
            Self::Process { stdin, acks, .. } => {
                send_process_command(stdin, acks, "expire");
            }
        }
    }

    pub fn add_peer_address(&self, peer_id: PeerId, addr: SocketAddr) {
        if let Self::Process { stdin, acks, .. } = self {
            send_process_command(stdin, acks, &format!("peer {peer_id} {addr}"));
        }
    }

    pub fn member_count(&self) -> usize {
        match self {
            Self::Task { node } => node.borrow_mut().member_count(),
            Self::Thread { snapshot, .. } => snapshot.lock().unwrap().member_count,
            Self::Process { snapshot, .. } => snapshot.lock().unwrap().member_count,
        }
    }

    pub fn peer_state(&self) -> PeerState {
        match self {
            Self::Task { node } => node.borrow_mut().peer_state(),
            Self::Thread { snapshot, .. } => snapshot.lock().unwrap().peer_state,
            Self::Process { snapshot, .. } => snapshot.lock().unwrap().peer_state,
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

    fn send_command(commands: &Sender<NodeCommand>, make: impl FnOnce(Sender<()>) -> NodeCommand) {
        let (ack, ack_rx) = channel::<()>();
        let _ = commands.send(make(ack));
        let _ = ack_rx.recv();
    }
}

fn send_process_command(stdin: &Mutex<ChildStdin>, acks: &Mutex<Receiver<()>>, line: &str) {
    if let Ok(mut stdin) = stdin.lock() {
        let _ = writeln!(stdin, "{line}");
        let _ = stdin.flush();
    }
    let _ = acks.lock().unwrap().recv();
}

fn spawn_stdout_reader(
    stdout: ChildStdout,
    snapshot: Arc<Mutex<NodeSnapshot>>,
    acks: Sender<()>,
) -> JoinHandle<()> {
    spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else {
                break;
            };
            if let Some(snap) = parse_state(&line) {
                *snapshot.lock().unwrap() = snap;
            } else if line.trim() == "ack" {
                let _ = acks.send(());
            }
        }
    })
}

fn parse_state(line: &str) -> Option<NodeSnapshot> {
    let mut parts = line.split_whitespace();
    if parts.next()? != "state" {
        return None;
    }
    let peer_state = parse_peer_state(parts.next()?);
    let member_count = parts.next()?.parse().ok()?;
    Some(NodeSnapshot {
        peer_state,
        member_count,
    })
}

fn parse_peer_state(name: &str) -> PeerState {
    match name {
        "Pinging" => PeerState::Pinging,
        "Collecting" => PeerState::Collecting,
        "Bootstrapped" => PeerState::Bootstrapped,
        "TimedOut" => PeerState::TimedOut,
        _ => PeerState::Fresh,
    }
}
