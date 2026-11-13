// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::quorum::Quorum;
use faction::types::PeerId;
use std::io::Write;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

use faction::reconfiguration::Reconfiguration;

use crate::constants::NODE_LIFETIME;
use crate::faction_node::FactionNode;
use crate::infra::network::address_book::AddressBook;
use crate::process_node::process_command::ProcessCommand;
use crate::time_ports::TimePorts;

pub struct ProcessRunner {
    node: FactionNode,
    commands: Receiver<ProcessCommand>,
    address_book: AddressBook,
    out: Box<dyn Write + Send>,
    time: TimePorts,
    idle_delay: Duration,
}

impl ProcessRunner {
    #[must_use]
    pub fn new(
        node: FactionNode,
        commands: Receiver<ProcessCommand>,
        address_book: AddressBook,
        out: Box<dyn Write + Send>,
        time: TimePorts,
        idle_delay: Duration,
    ) -> Self {
        Self {
            node,
            commands,
            address_book,
            out,
            time,
            idle_delay,
        }
    }

    pub fn run(mut self) {
        self.node.start();
        self.emit_state();
        let deadline = self.time.clock.now() + NODE_LIFETIME;
        while self.drain_commands() {
            let had_work = self.node.step();
            self.emit_state();
            if self.time.clock.now() >= deadline {
                break;
            }
            if !had_work {
                self.time.pause.pause(self.idle_delay);
            }
        }
        self.emit_state();
    }

    // The change in flight as the state line writes it: none, an addition
    // requested of a peer or acknowledged by it, or a peer's removal.
    fn addition(addition: Reconfiguration) -> String {
        match addition {
            Reconfiguration::None => String::from("none"),
            Reconfiguration::AdditionRequested { peer_id } => format!("requested:{peer_id}"),
            Reconfiguration::AdditionAcknowledged { peer_id } => format!("acknowledged:{peer_id}"),
            Reconfiguration::RemovalRequested { peer_id } => format!("removing:{peer_id}"),
        }
    }

    fn apply(&mut self, command: ProcessCommand) {
        match command {
            ProcessCommand::RequestJoin(peer_id) => self.node.request_join(peer_id),
            ProcessCommand::Admit(peer_id) => self.node.admit(peer_id),
            ProcessCommand::Deny(peer_id) => self.node.deny(peer_id),
            ProcessCommand::ExpireDeadline => self.node.expire_deadline(),
            ProcessCommand::RequestAddition(peer_id) => self.node.request_addition(peer_id),
            ProcessCommand::CommitAddition(peer_id) => self.node.commit_addition(peer_id),
            ProcessCommand::AbortAddition(peer_id) => self.node.abort_addition(peer_id),
            ProcessCommand::RequiredCount(required_count) => {
                self.node.change_required_count(required_count);
            }
            ProcessCommand::Cut(peer_id) => self.node.cut(peer_id),
            ProcessCommand::Mend(peer_id) => self.node.mend(peer_id),
            ProcessCommand::Peer(peer_id, addr) => {
                self.address_book.lock().unwrap().push((peer_id, addr));
            }
            ProcessCommand::Shutdown => {}
        }
    }

    fn drain_commands(&mut self) -> bool {
        loop {
            match self.commands.try_recv() {
                Ok(ProcessCommand::Shutdown) | Err(TryRecvError::Disconnected) => return false,
                Ok(command) => {
                    self.apply(command);
                    self.emit_state();
                    self.emit_ack();
                }
                Err(TryRecvError::Empty) => return true,
            }
        }
    }

    fn emit_ack(&mut self) {
        let _ = writeln!(self.out, "ack");
        let _ = self.out.flush();
    }

    fn emit_state(&mut self) {
        let snapshot = self.node.snapshot();
        let _ = writeln!(
            self.out,
            "state {:?} {} {} {} {} {}",
            snapshot.peer_state,
            snapshot.member_count,
            snapshot.required_count,
            Self::addition(snapshot.addition),
            Self::suspected(&snapshot.suspected),
            Self::quorum(snapshot.quorum)
        );
        let _ = self.out.flush();
    }

    // The quorum as the state line writes it.
    fn quorum(quorum: Quorum) -> &'static str {
        match quorum {
            Quorum::NotTracked => "off",
            Quorum::Held => "held",
            Quorum::Lost => "lost",
        }
    }

    // The suspected members as the state line writes them: by id, comma
    // separated, or a dash for none.
    fn suspected(suspected: &[PeerId]) -> String {
        if suspected.is_empty() {
            String::from("-")
        } else {
            suspected
                .iter()
                .map(PeerId::to_string)
                .collect::<Vec<String>>()
                .join(",")
        }
    }
}
