// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::quorum::Quorum;
use std::io::BufRead;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::thread::spawn;

use faction::peer_state::PeerState;
use faction::reconfiguration::Reconfiguration;

use crate::infra::proposal::Proposal;
use crate::node_snapshot::NodeSnapshot;

pub struct StateReader {
    snapshot: Arc<Mutex<NodeSnapshot>>,
    proposals: Arc<Mutex<Vec<Proposal>>>,
    acks: Sender<()>,
}

impl StateReader {
    #[must_use]
    pub fn new(
        snapshot: Arc<Mutex<NodeSnapshot>>,
        proposals: Arc<Mutex<Vec<Proposal>>>,
        acks: Sender<()>,
    ) -> Self {
        Self {
            snapshot,
            proposals,
            acks,
        }
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

    fn parse_peers(text: &str) -> Option<Vec<u64>> {
        if text == "-" {
            Some(Vec::new())
        } else {
            text.split(',').map(|peer| peer.parse().ok()).collect()
        }
    }

    fn parse_proposal(line: &str) -> Option<Proposal> {
        let mut parts = line.split_whitespace();
        if parts.next()? != "proposal" {
            return None;
        }
        let kind = parts.next()?;
        let peer_id = parts.next()?.parse().ok()?;
        let epoch = parts.next()?.parse().ok()?;
        match kind {
            "addition" => Some(Proposal::Addition { peer_id, epoch }),
            "removal" => Some(Proposal::Removal { peer_id, epoch }),
            _ => None,
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
            _ => None,
        }
    }

    fn parse_reconfigurations(text: &str) -> Option<Vec<Reconfiguration>> {
        if text == "none" {
            Some(Vec::new())
        } else {
            text.split(',').map(Self::parse_reconfiguration).collect()
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
        let reconfigurations = Self::parse_reconfigurations(parts.next()?)?;
        let suspected = Self::parse_peers(parts.next()?)?;
        let quorum = Self::parse_quorum(parts.next()?)?;
        let epoch = parts.next()?.parse().ok()?;
        let members = Self::parse_peers(parts.next()?)?;
        Some(NodeSnapshot {
            peer_state,
            member_count,
            members,
            required_count,
            reconfigurations,
            suspected,
            quorum,
            epoch,
        })
    }

    fn read(&self, line: &str) {
        if let Some(snapshot) = Self::parse_state(line) {
            *self.snapshot.lock().unwrap() = snapshot;
        } else if let Some(proposal) = Self::parse_proposal(line) {
            self.proposals.lock().unwrap().push(proposal);
        } else if line.trim() == "ack" {
            let _ = self.acks.send(());
        }
    }
}
