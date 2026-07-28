// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::io::{BufRead, Write, stdin, stdout};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

use faction::types::PeerId;

use crate::faction_node::FactionNode;

pub type AddressBook = Arc<Mutex<Vec<(PeerId, SocketAddr)>>>;

enum ProcessCommand {
    RequestJoin(PeerId),
    Admit(PeerId),
    Deny(PeerId),
    ExpireDeadline,
    Peer(PeerId, SocketAddr),
    Shutdown,
}

fn parse(line: &str) -> Option<ProcessCommand> {
    let mut parts = line.split_whitespace();
    match parts.next()? {
        "request-join" => Some(ProcessCommand::RequestJoin(parts.next()?.parse().ok()?)),
        "admit" => Some(ProcessCommand::Admit(parts.next()?.parse().ok()?)),
        "deny" => Some(ProcessCommand::Deny(parts.next()?.parse().ok()?)),
        "expire" => Some(ProcessCommand::ExpireDeadline),
        "peer" => {
            let id = parts.next()?.parse().ok()?;
            let addr = parts.next()?.parse().ok()?;
            Some(ProcessCommand::Peer(id, addr))
        }
        "shutdown" => Some(ProcessCommand::Shutdown),
        _ => None,
    }
}

fn spawn_stdin_reader() -> Receiver<ProcessCommand> {
    let (tx, rx) = channel();
    spawn(move || {
        for line in stdin().lock().lines() {
            let Ok(line) = line else {
                break;
            };
            if let Some(command) = parse(&line) {
                if tx.send(command).is_err() {
                    break;
                }
            }
        }
    });
    rx
}

pub fn run(mut node: FactionNode, address_book: AddressBook, delay: Duration) {
    let commands = spawn_stdin_reader();
    let mut out = stdout();

    node.start();
    emit_state(&mut out, &mut node);

    let deadline = Instant::now() + Duration::from_secs(30);
    'outer: loop {
        loop {
            match commands.try_recv() {
                Ok(ProcessCommand::Shutdown) | Err(TryRecvError::Disconnected) => break 'outer,
                Ok(command) => {
                    apply(&mut node, &address_book, command);
                    emit_state(&mut out, &mut node);
                    emit_ack(&mut out);
                }
                Err(TryRecvError::Empty) => break,
            }
        }

        let had_work = node.step();
        emit_state(&mut out, &mut node);
        if Instant::now() >= deadline {
            break;
        }
        if !had_work {
            sleep(delay);
        }
    }
    emit_state(&mut out, &mut node);
}

fn apply(node: &mut FactionNode, address_book: &AddressBook, command: ProcessCommand) {
    match command {
        ProcessCommand::RequestJoin(peer_id) => node.request_join(peer_id),
        ProcessCommand::Admit(peer_id) => node.admit(peer_id),
        ProcessCommand::Deny(peer_id) => node.deny(peer_id),
        ProcessCommand::ExpireDeadline => node.expire_deadline(),
        ProcessCommand::Peer(peer_id, addr) => address_book.lock().unwrap().push((peer_id, addr)),
        ProcessCommand::Shutdown => {}
    }
}

fn emit_state(out: &mut impl Write, node: &mut FactionNode) {
    let _ = writeln!(out, "state {:?} {}", node.peer_state(), node.member_count());
    let _ = out.flush();
}

fn emit_ack(out: &mut impl Write) {
    let _ = writeln!(out, "ack");
    let _ = out.flush();
}
