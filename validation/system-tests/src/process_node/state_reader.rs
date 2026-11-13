// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::io::BufRead;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::thread::spawn;

use faction::peer_state::PeerState;

use crate::node_snapshot::NodeSnapshot;

pub struct StateReader {
    snapshot: Arc<Mutex<NodeSnapshot>>,
    acks: Sender<()>,
}

impl StateReader {
    #[must_use]
    pub fn new(snapshot: Arc<Mutex<NodeSnapshot>>, acks: Sender<()>) -> Self {
        Self { snapshot, acks }
    }

    pub fn spawn(self, output: impl BufRead + Send + 'static) -> JoinHandle<()> {
        spawn(move || {
            for line in output.lines() {
                let Ok(line) = line else {
                    break;
                };
                self.read(&line);
            }
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

    fn parse_state(line: &str) -> Option<NodeSnapshot> {
        let mut parts = line.split_whitespace();
        if parts.next()? != "state" {
            return None;
        }
        let peer_state = Self::parse_peer_state(parts.next()?);
        let member_count = parts.next()?.parse().ok()?;
        Some(NodeSnapshot {
            peer_state,
            member_count,
        })
    }

    fn read(&self, line: &str) {
        if let Some(snapshot) = Self::parse_state(line) {
            *self.snapshot.lock().unwrap() = snapshot;
        } else if line.trim() == "ack" {
            let _ = self.acks.send(());
        }
    }
}
