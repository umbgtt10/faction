// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use faction::cluster_view::ClusterView;
use faction::peer_state::PeerState;
use faction::types::PeerId;

pub struct ClusterInvariants {
    views: Vec<ClusterView>,
}

impl ClusterInvariants {
    #[must_use]
    pub fn new(views: Vec<ClusterView>) -> Self {
        Self { views }
    }

    #[must_use]
    pub fn distinct(&self) -> Vec<String> {
        self.counted()
            .flat_map(|(node, peers)| {
                peers
                    .iter()
                    .enumerate()
                    .filter(|(position, peer)| peers[..*position].contains(peer))
                    .map(move |(_, peer)| format!("node {node} counts peer {peer} twice"))
            })
            .collect()
    }

    #[must_use]
    pub fn membership(&self) -> Vec<String> {
        self.views
            .iter()
            .enumerate()
            .flat_map(|(node, view)| {
                view.pinging_peers()
                    .iter()
                    .chain(view.collecting_peers())
                    .filter(|peer| !view.members().is_member(**peer))
                    .map(move |peer| format!("node {node} counts peer {peer}, not a member"))
            })
            .collect()
    }

    #[must_use]
    pub fn quorum(&self) -> Vec<String> {
        self.views
            .iter()
            .enumerate()
            .filter(|(_, view)| {
                view.peer_state() == PeerState::Bootstrapped
                    && view.collecting_peers().len() < view.required_count()
            })
            .map(|(node, view)| {
                format!(
                    "node {node} concluded with {} of {} confirmations",
                    view.collecting_peers().len(),
                    view.required_count()
                )
            })
            .collect()
    }

    #[must_use]
    pub fn violations(&self) -> Vec<String> {
        let mut violations = self.distinct();
        violations.extend(self.membership());
        violations.extend(self.quorum());
        violations
    }

    fn counted(&self) -> impl Iterator<Item = (usize, &[PeerId])> {
        self.views.iter().enumerate().flat_map(|(node, view)| {
            [
                (node, view.pinging_peers()),
                (node, view.collecting_peers()),
            ]
        })
    }
}
