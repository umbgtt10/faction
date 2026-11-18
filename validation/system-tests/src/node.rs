// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::quorum::Quorum;
use std::cell::RefCell;
use std::io::BufReader;
use std::mem::take;
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
use faction::types::Epoch;
use serde_json::to_string;

use crate::faction_node::FactionNode;
use crate::infra::fault_settings::FaultSettings;
use crate::infra::proposal::Proposal;
use crate::membership::Membership;
use crate::node_command::NodeCommand;
use crate::node_snapshot::NodeSnapshot;
use crate::process_node::process_channel::ProcessChannel;
use crate::process_node::state_reader::StateReader;
use crate::thread_runner::ThreadRunner;
use crate::time_ports::TimePorts;

const FRESH: NodeSnapshot = NodeSnapshot {
    peer_state: PeerState::Fresh,
    member_count: 0,
    members: Vec::new(),
    required_count: 0,
    reconfigurations: Vec::new(),
    suspected: Vec::new(),
    quorum: Quorum::NotTracked,
    epoch: 0,
};

pub enum Node {
    Task {
        node: Rc<RefCell<FactionNode>>,
    },
    Thread {
        snapshot: Arc<Mutex<NodeSnapshot>>,
        proposals: Arc<Mutex<Vec<Proposal>>>,
        commands: Sender<NodeCommand>,
        handle: Option<JoinHandle<()>>,
    },
    Process {
        child: Mutex<Child>,
        channel: ProcessChannel,
        snapshot: Arc<Mutex<NodeSnapshot>>,
        proposals: Arc<Mutex<Vec<Proposal>>>,
        reader: Option<JoinHandle<()>>,
    },
}

impl Node {
    #[must_use]
    pub fn process(mut child: Child) -> Self {
        let stdin = child.stdin.take().expect("child stdin is piped");
        let stdout = child.stdout.take().expect("child stdout is piped");
        let snapshot = Arc::new(Mutex::new(FRESH));
        let proposals = Arc::new(Mutex::new(Vec::new()));
        let (acks, ack_receiver) = channel();
        let reader = StateReader::new(snapshot.clone(), proposals.clone(), acks)
            .spawn(BufReader::new(stdout));
        Self::Process {
            child: Mutex::new(child),
            channel: ProcessChannel::new(Box::new(stdin), ack_receiver),
            snapshot,
            proposals,
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
        lifetime: Duration,
    ) -> Self {
        let snapshot = Arc::new(Mutex::new(FRESH));
        let thread_snapshot = snapshot.clone();
        let proposals = Arc::new(Mutex::new(Vec::new()));
        let thread_proposals = proposals.clone();
        let (commands, command_receiver) = channel();
        let handle = spawn(move || {
            ThreadRunner::new(
                build(),
                command_receiver,
                thread_snapshot,
                thread_proposals,
                time,
                idle_delay,
                lifetime,
            )
            .run();
        });
        Self::Thread {
            snapshot,
            proposals,
            commands,
            handle: Some(handle),
        }
    }

    pub fn abort_addition(&self, peer_id: u64) {
        match self {
            Self::Task { node } => node.borrow_mut().abort_addition(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::AbortAddition(peer_id, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("abort-addition {peer_id}")),
        }
    }

    pub fn abort_removal(&self, peer_id: u64) {
        match self {
            Self::Task { node } => node.borrow_mut().abort_removal(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::AbortRemoval(peer_id, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("abort-removal {peer_id}")),
        }
    }

    pub fn add_peer_address(&self, peer_id: u64, addr: SocketAddr) {
        if let Self::Process { channel, .. } = self {
            channel.send(&format!("peer {peer_id} {addr}"));
        }
    }

    pub fn admit(&self, peer_id: u64) {
        match self {
            Self::Task { node } => node.borrow_mut().admit(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::Admit(peer_id, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("admit {peer_id}")),
        }
    }

    pub fn change_required_count(&self, required_count: usize, epoch: Epoch) {
        match self {
            Self::Task { node } => node
                .borrow_mut()
                .change_required_count(required_count, epoch),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| {
                    NodeCommand::ChangeRequiredCount(required_count, epoch, ack)
                });
            }
            Self::Process { channel, .. } => {
                channel.send(&format!("required-count {required_count} {epoch}"))
            }
        }
    }

    pub fn commit_addition(&self, peer_id: u64, epoch: Epoch) {
        match self {
            Self::Task { node } => node.borrow_mut().commit_addition(peer_id, epoch),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| {
                    NodeCommand::CommitAddition(peer_id, epoch, ack)
                });
            }
            Self::Process { channel, .. } => {
                channel.send(&format!("commit-addition {peer_id} {epoch}"));
            }
        }
    }

    pub fn commit_removal(&self, peer_id: u64, epoch: Epoch) {
        match self {
            Self::Task { node } => node.borrow_mut().commit_removal(peer_id, epoch),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| {
                    NodeCommand::CommitRemoval(peer_id, epoch, ack)
                });
            }
            Self::Process { channel, .. } => {
                channel.send(&format!("commit-removal {peer_id} {epoch}"));
            }
        }
    }

    pub fn cut(&self, peer_id: u64) {
        match self {
            Self::Task { node } => node.borrow_mut().cut(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::Cut(peer_id, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("cut {peer_id}")),
        }
    }

    pub fn decline(&self, peer_id: u64, epoch: Epoch) {
        match self {
            Self::Task { node } => node.borrow_mut().decline(peer_id, epoch),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::Decline(peer_id, epoch, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("decline {peer_id} {epoch}")),
        }
    }

    pub fn deny(&self, peer_id: u64) {
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

    pub fn mend(&self, peer_id: u64) {
        match self {
            Self::Task { node } => node.borrow_mut().mend(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::Mend(peer_id, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("mend {peer_id}")),
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

    pub fn request_addition(&self, peer_id: u64) {
        match self {
            Self::Task { node } => node.borrow_mut().request_addition(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::RequestAddition(peer_id, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("request-addition {peer_id}")),
        }
    }

    pub fn request_join(&self, peer_id: u64) {
        match self {
            Self::Task { node } => node.borrow_mut().request_join(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::RequestJoin(peer_id, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("request-join {peer_id}")),
        }
    }

    pub fn request_removal(&self, peer_id: u64) {
        match self {
            Self::Task { node } => node.borrow_mut().request_removal(peer_id),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::RequestRemoval(peer_id, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!("request-removal {peer_id}")),
        }
    }

    // The node created again at `membership`, in place: its state gone, its
    // link kept. A process takes it as a line: the epoch, the count, then the
    // peers comma separated.
    pub fn restart(&self, membership: Membership) {
        match self {
            Self::Task { node } => node.borrow_mut().restart(&membership),
            Self::Thread { commands, .. } => {
                Self::send_command(commands, |ack| NodeCommand::Restart(membership, ack));
            }
            Self::Process { channel, .. } => {
                let peers: Vec<String> = membership.peers.iter().map(u64::to_string).collect();
                channel.send(&format!(
                    "restart {} {} {}",
                    membership.epoch,
                    membership.required_count,
                    peers.join(",")
                ));
            }
        }
    }

    // The node's faults set to `settings`: in place for a task's node, by
    // command for a thread's, and on its command stream, as JSON, for a
    // process's.
    pub fn set_faults(&self, settings: &FaultSettings) {
        match self {
            Self::Task { node } => node.borrow_mut().set_faults(settings),
            Self::Thread { commands, .. } => {
                let settings = Box::new(settings.clone());
                Self::send_command(commands, |ack| NodeCommand::SetFaults(settings, ack));
            }
            Self::Process { channel, .. } => channel.send(&format!(
                "faults {}",
                to_string(settings).expect("fault settings written as JSON")
            )),
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

    #[must_use]
    pub fn snapshot(&self) -> NodeSnapshot {
        match self {
            Self::Task { node } => node.borrow_mut().snapshot(),
            Self::Thread { snapshot, .. } | Self::Process { snapshot, .. } => {
                snapshot.lock().unwrap().clone()
            }
        }
    }

    pub fn start(&self) {
        if let Self::Task { node } = self {
            node.borrow_mut().start();
        }
    }

    // One input of a task node's taken, if it had one: whether it had. A
    // thread's or a process's node steps on its own.
    pub fn step(&self) -> bool {
        match self {
            Self::Task { node } => node.borrow_mut().step(),
            Self::Thread { .. } | Self::Process { .. } => false,
        }
    }

    // What the node proposed since it was last asked, in the order it
    // proposed it.
    #[must_use]
    pub fn take_proposals(&self) -> Vec<Proposal> {
        match self {
            Self::Task { node } => node.borrow_mut().take_proposals(),
            Self::Thread { proposals, .. } | Self::Process { proposals, .. } => {
                take(&mut *proposals.lock().unwrap())
            }
        }
    }

    fn send_command(commands: &Sender<NodeCommand>, make: impl FnOnce(Sender<()>) -> NodeCommand) {
        let (ack, ack_receiver) = channel::<()>();
        let _ = commands.send(make(ack));
        let _ = ack_receiver.recv();
    }
}
