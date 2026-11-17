// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::types::Epoch;
use faction::types::PeerId;
use faction_protocol::commit::Commit;
use faction_system_tests::node_snapshot::NodeSnapshot;

use crate::harness::committed_log::CommittedLog;

// The end state the committed log implies, computed, never observed: every
// member concluded at the log's membership -- its members, its required
// count, its last epoch -- with no change in flight, no suspect, and its
// quorum held; every node the log removed and never took back `Removed`, at
// the epoch of its removal; and a newcomer it never added still waiting.
pub struct ExpectedEnd {
    member: NodeSnapshot,
    named: Vec<PeerId>,
    removals: Vec<(PeerId, Epoch)>,
}

impl ExpectedEnd {
    #[must_use]
    pub fn new(log: &CommittedLog) -> Self {
        let membership = log.membership();
        let mut members = membership.peers.clone();
        members.sort_unstable();
        Self {
            member: NodeSnapshot {
                peer_state: PeerState::Bootstrapped,
                member_count: members.len(),
                members,
                required_count: membership.required_count,
                reconfigurations: Vec::new(),
                suspected: Vec::new(),
                quorum: Quorum::Held,
                epoch: membership.epoch,
            },
            named: log
                .membership_at(0)
                .peers
                .into_iter()
                .chain(log.commits().iter().filter_map(|commit| match *commit {
                    Commit::Addition { peer_id, .. } => Some(peer_id),
                    _ => None,
                }))
                .collect(),
            removals: log
                .commits()
                .iter()
                .filter_map(|commit| match *commit {
                    Commit::Removal { peer_id, .. } => {
                        log.removed_at(peer_id).map(|epoch| (peer_id, epoch))
                    }
                    _ => None,
                })
                .collect(),
        }
    }

    // What the node `peer_id`, as `snapshot` shows it, was expected to show,
    // if it shows something else.
    #[must_use]
    pub fn departure(&self, peer_id: PeerId, snapshot: &NodeSnapshot) -> Option<String> {
        if let Some(&(_, epoch)) = self
            .removals
            .iter()
            .find(|(removed, _)| *removed == peer_id)
        {
            let removed = snapshot.peer_state == PeerState::Removed && snapshot.epoch == epoch;
            return (!removed).then(|| format!("Removed at epoch {epoch}"));
        }
        let expected = if self.named.contains(&peer_id) {
            self.member.clone()
        } else {
            Self::waiting()
        };
        let mut found = snapshot.clone();
        found.members.sort_unstable();
        (found != expected).then(|| format!("{expected:?}"))
    }

    // A newcomer that has received no membership.
    fn waiting() -> NodeSnapshot {
        NodeSnapshot {
            peer_state: PeerState::Fresh,
            member_count: 0,
            members: Vec::new(),
            required_count: 0,
            reconfigurations: Vec::new(),
            suspected: Vec::new(),
            quorum: Quorum::NotTracked,
            epoch: 0,
        }
    }
}
