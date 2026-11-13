// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::io::Write;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

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

    fn apply(&mut self, command: ProcessCommand) {
        match command {
            ProcessCommand::RequestJoin(peer_id) => self.node.request_join(peer_id),
            ProcessCommand::Admit(peer_id) => self.node.admit(peer_id),
            ProcessCommand::Deny(peer_id) => self.node.deny(peer_id),
            ProcessCommand::ExpireDeadline => self.node.expire_deadline(),
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
            "state {:?} {}",
            snapshot.peer_state, snapshot.member_count
        );
        let _ = self.out.flush();
    }
}
