// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::Epoch;
use faction::types::PeerId;
use faction_protocol::commit::Commit;
use faction_system_tests::membership::Membership;

use crate::model::change::Change;

// The changes the layer above has committed, numbered from one in the order
// it committed them, over the genesis membership they change. A node at an
// epoch holds the commits up to it, and lacks those after.
pub struct CommittedLog {
    genesis: Membership,
    commits: Vec<Commit>,
}

impl CommittedLog {
    #[must_use]
    pub fn new(genesis: Membership) -> Self {
        Self {
            genesis,
            commits: Vec::new(),
        }
    }

    // The place of the first commit a node at `epoch` lacks.
    #[must_use]
    pub fn after(&self, epoch: Epoch) -> usize {
        usize::try_from(epoch)
            .unwrap_or(usize::MAX)
            .min(self.commits.len())
    }

    // The change committed, at the next epoch.
    pub fn commit(&mut self, change: Change) -> Commit {
        let epoch = self.commits.len() as Epoch + 1;
        let commit = match change {
            Change::Addition { peer_id } => Commit::Addition { peer_id, epoch },
            Change::Removal { peer_id } => Commit::Removal { peer_id, epoch },
            Change::RequiredCount { required_count } => Commit::RequiredCount {
                required_count,
                epoch,
            },
        };
        self.commits.push(commit);
        commit
    }

    #[must_use]
    pub fn commits(&self) -> &[Commit] {
        &self.commits
    }

    #[must_use]
    pub fn membership(&self) -> Membership {
        self.membership_at(self.commits.len())
    }

    // The membership the first `prefix` commits make of the genesis one.
    #[must_use]
    pub fn membership_at(&self, prefix: usize) -> Membership {
        self.commits[..prefix.min(self.commits.len())]
            .iter()
            .fold(self.genesis.clone(), Self::applied)
    }

    // The epoch `peer_id` was last removed at, if the log leaves it removed.
    #[must_use]
    pub fn removed_at(&self, peer_id: PeerId) -> Option<Epoch> {
        if self.membership().peers.contains(&peer_id) {
            return None;
        }
        self.commits.iter().rev().find_map(|commit| match commit {
            Commit::Removal {
                peer_id: removed,
                epoch,
            } if *removed == peer_id => Some(*epoch),
            _ => None,
        })
    }

    fn applied(membership: Membership, commit: &Commit) -> Membership {
        let mut peers = membership.peers;
        let required_count = match *commit {
            Commit::Addition { peer_id, .. } => {
                peers.push(peer_id);
                membership.required_count
            }
            Commit::Removal { peer_id, .. } => {
                peers.retain(|peer| *peer != peer_id);
                membership.required_count
            }
            Commit::RequiredCount { required_count, .. } => required_count,
        };
        Membership {
            peers,
            required_count,
            epoch: membership.epoch + 1,
        }
    }
}
