// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::quorum::Quorum;
use faction::types::PeerId;
use std::io::BufRead;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::thread::spawn;

use faction::peer_state::PeerState;
use faction::reconfiguration::Reconfiguration;

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
            "Removed" => PeerState::Removed,
            _ => PeerState::Fresh,
        }
    }

    fn parse_quorum(text: &str) -> Option<Quorum> {
        match text {
            "off" => Some(Quorum::NotTracked),
            "held" => Some(Quorum::Held),
            "lost" => Some(Quorum::Lost),
            _ => None,
        }
    }

    fn parse_reconfiguration(text: &str) -> Option<Reconfiguration> {
        match text.split_once(':') {
            Some(("requested", peer_id)) => Some(Reconfiguration::AdditionRequested {
                peer_id: peer_id.parse().ok()?,
            }),
            Some(("acknowledged", peer_id)) => Some(Reconfiguration::AdditionAcknowledged {
                peer_id: peer_id.parse().ok()?,
            }),
            Some(("removing", peer_id)) => Some(Reconfiguration::RemovalRequested {
                peer_id: peer_id.parse().ok()?,
            }),
            _ => (text == "none").then_some(Reconfiguration::None),
        }
    }

    fn parse_state(line: &str) -> Option<NodeSnapshot> {
        let mut parts = line.split_whitespace();
        if parts.next()? != "state" {
            return None;
        }
        let peer_state = Self::parse_peer_state(parts.next()?);
        let member_count = parts.next()?.parse().ok()?;
        let required_count = parts.next()?.parse().ok()?;
        let reconfiguration = Self::parse_reconfiguration(parts.next()?)?;
        let suspected = Self::parse_suspected(parts.next()?)?;
        let quorum = Self::parse_quorum(parts.next()?)?;
        Some(NodeSnapshot {
            peer_state,
            member_count,
            required_count,
            reconfiguration,
            suspected,
            quorum,
        })
    }

    fn parse_suspected(text: &str) -> Option<Vec<PeerId>> {
        if text == "-" {
            Some(Vec::new())
        } else {
            text.split(',').map(|peer| peer.parse().ok()).collect()
        }
    }

    fn read(&self, line: &str) {
        if let Some(snapshot) = Self::parse_state(line) {
            *self.snapshot.lock().unwrap() = snapshot;
        } else if line.trim() == "ack" {
            let _ = self.acks.send(());
        }
    }
}
