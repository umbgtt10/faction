// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::io::BufRead;
use std::net::SocketAddr;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use std::thread::spawn;

use faction::types::Epoch;
use faction::types::PeerId;

use serde_json::from_str;

use crate::infra::fault_settings::FaultSettings;
use crate::membership::Membership;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessCommand {
    RequestJoin(PeerId),
    Admit(PeerId),
    Deny(PeerId),
    ExpireDeadline,
    RequestAddition(PeerId),
    CommitAddition(PeerId, Epoch),
    AbortAddition(PeerId),
    RequestRemoval(PeerId),
    CommitRemoval(PeerId, Epoch),
    AbortRemoval(PeerId),
    RequiredCount(usize, Epoch),
    Cut(PeerId),
    Mend(PeerId),
    Peer(PeerId, SocketAddr),
    Restart(Membership),
    Faults(Box<FaultSettings>),
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
            "request-addition" => Some(Self::RequestAddition(parts.next()?.parse().ok()?)),
            "commit-addition" => Some(Self::CommitAddition(
                parts.next()?.parse().ok()?,
                parts.next()?.parse().ok()?,
            )),
            "abort-addition" => Some(Self::AbortAddition(parts.next()?.parse().ok()?)),
            "request-removal" => Some(Self::RequestRemoval(parts.next()?.parse().ok()?)),
            "commit-removal" => Some(Self::CommitRemoval(
                parts.next()?.parse().ok()?,
                parts.next()?.parse().ok()?,
            )),
            "abort-removal" => Some(Self::AbortRemoval(parts.next()?.parse().ok()?)),
            "required-count" => Some(Self::RequiredCount(
                parts.next()?.parse().ok()?,
                parts.next()?.parse().ok()?,
            )),
            "cut" => Some(Self::Cut(parts.next()?.parse().ok()?)),
            "mend" => Some(Self::Mend(parts.next()?.parse().ok()?)),
            "peer" => {
                let id = parts.next()?.parse().ok()?;
                let addr = parts.next()?.parse().ok()?;
                Some(Self::Peer(id, addr))
            }
            "restart" => Some(Self::Restart(Membership {
                epoch: parts.next()?.parse().ok()?,
                required_count: parts.next()?.parse().ok()?,
                peers: parts
                    .next()?
                    .split(',')
                    .map(|peer| peer.parse().ok())
                    .collect::<Option<Vec<PeerId>>>()?,
            })),
            "faults" => from_str(line.split_once(' ')?.1)
                .ok()
                .map(|settings| Self::Faults(Box::new(settings))),
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
