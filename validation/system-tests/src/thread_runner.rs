// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

use crate::faction_node::FactionNode;
use crate::infra::proposal::Proposal;
use crate::node_command::NodeCommand;
use crate::node_snapshot::NodeSnapshot;
use crate::time_ports::TimePorts;

pub struct ThreadRunner {
    node: FactionNode,
    commands: Receiver<NodeCommand>,
    snapshot: Arc<Mutex<NodeSnapshot>>,
    proposals: Arc<Mutex<Vec<Proposal>>>,
    time: TimePorts,
    idle_delay: Duration,
    lifetime: Duration,
}

impl ThreadRunner {
    #[must_use]
    pub fn new(
        node: FactionNode,
        commands: Receiver<NodeCommand>,
        snapshot: Arc<Mutex<NodeSnapshot>>,
        proposals: Arc<Mutex<Vec<Proposal>>>,
        time: TimePorts,
        idle_delay: Duration,
        lifetime: Duration,
    ) -> Self {
        Self {
            node,
            commands,
            snapshot,
            proposals,
            time,
            idle_delay,
            lifetime,
        }
    }

    pub fn run(mut self) {
        self.node.start();
        self.publish();
        let deadline = self.time.clock.now() + self.lifetime;
        while self.drain_commands() {
            let had_work = self.node.step();
            self.publish();
            if self.time.clock.now() >= deadline {
                break;
            }
            if !had_work {
                self.time.pause.pause(self.idle_delay);
            }
        }
        self.publish();
    }

    fn drain_commands(&mut self) -> bool {
        loop {
            let ack = match self.commands.try_recv() {
                Ok(NodeCommand::RequestJoin(peer_id, ack)) => {
                    self.node.request_join(peer_id);
                    ack
                }
                Ok(NodeCommand::Admit(peer_id, ack)) => {
                    self.node.admit(peer_id);
                    ack
                }
                Ok(NodeCommand::Deny(peer_id, ack)) => {
                    self.node.deny(peer_id);
                    ack
                }
                Ok(NodeCommand::ExpireDeadline(ack)) => {
                    self.node.expire_deadline();
                    ack
                }
                Ok(NodeCommand::RequestAddition(peer_id, ack)) => {
                    self.node.request_addition(peer_id);
                    ack
                }
                Ok(NodeCommand::CommitAddition(peer_id, epoch, ack)) => {
                    self.node.commit_addition(peer_id, epoch);
                    ack
                }
                Ok(NodeCommand::AbortAddition(peer_id, ack)) => {
                    self.node.abort_addition(peer_id);
                    ack
                }
                Ok(NodeCommand::RequestRemoval(peer_id, ack)) => {
                    self.node.request_removal(peer_id);
                    ack
                }
                Ok(NodeCommand::CommitRemoval(peer_id, epoch, ack)) => {
                    self.node.commit_removal(peer_id, epoch);
                    ack
                }
                Ok(NodeCommand::AbortRemoval(peer_id, ack)) => {
                    self.node.abort_removal(peer_id);
                    ack
                }
                Ok(NodeCommand::Decline(peer_id, epoch, ack)) => {
                    self.node.decline(peer_id, epoch);
                    ack
                }
                Ok(NodeCommand::Cut(peer_id, ack)) => {
                    self.node.cut(peer_id);
                    ack
                }
                Ok(NodeCommand::Mend(peer_id, ack)) => {
                    self.node.mend(peer_id);
                    ack
                }
                Ok(NodeCommand::ChangeRequiredCount(required_count, epoch, ack)) => {
                    self.node.change_required_count(required_count, epoch);
                    ack
                }
                Ok(NodeCommand::Restart(membership, ack)) => {
                    self.node.restart(&membership);
                    ack
                }
                Ok(NodeCommand::SetFaults(settings, ack)) => {
                    self.node.set_faults(&settings);
                    ack
                }
                Ok(NodeCommand::Shutdown) | Err(TryRecvError::Disconnected) => return false,
                Err(TryRecvError::Empty) => return true,
            };
            self.publish();
            let _ = ack.send(());
        }
    }

    // The node's state, and what it proposed, where the layer above reads them.
    fn publish(&mut self) {
        *self.snapshot.lock().unwrap() = self.node.snapshot();
        self.proposals
            .lock()
            .unwrap()
            .extend(self.node.take_proposals());
    }
}
