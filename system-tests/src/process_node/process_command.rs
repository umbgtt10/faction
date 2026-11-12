// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::io::BufRead;
use std::net::SocketAddr;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use std::thread::spawn;

use faction::types::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessCommand {
    RequestJoin(PeerId),
    Admit(PeerId),
    Deny(PeerId),
    ExpireDeadline,
    Peer(PeerId, SocketAddr),
    Shutdown,
}

impl ProcessCommand {
    #[must_use]
    pub fn parse(line: &str) -> Option<Self> {
        let mut parts = line.split_whitespace();
        match parts.next()? {
            "request-join" => Some(Self::RequestJoin(parts.next()?.parse().ok()?)),
            "admit" => Some(Self::Admit(parts.next()?.parse().ok()?)),
            "deny" => Some(Self::Deny(parts.next()?.parse().ok()?)),
            "expire" => Some(Self::ExpireDeadline),
            "peer" => {
                let id = parts.next()?.parse().ok()?;
                let addr = parts.next()?.parse().ok()?;
                Some(Self::Peer(id, addr))
            }
            "shutdown" => Some(Self::Shutdown),
            _ => None,
        }
    }

    pub fn read_lines(input: impl BufRead + Send + 'static) -> Receiver<Self> {
        let (sender, receiver) = channel();
        spawn(move || {
            for line in input.lines() {
                let Ok(line) = line else {
                    break;
                };
                if let Some(command) = Self::parse(&line) {
                    if sender.send(command).is_err() {
                        break;
                    }
                }
            }
        });
        receiver
    }
}
