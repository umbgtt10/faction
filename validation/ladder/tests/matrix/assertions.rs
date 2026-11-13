// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::faction::Faction;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum::Quorum;

#[derive(Debug, Clone, Copy)]
pub enum Assert {
    PingingCount(usize),
    CollectingCount(usize),
    Exited,
    NotExited,
    Conclusion(Conclusion),
    LocalComplete,
    NotLocalComplete,
    Quorum(Quorum),
    SuspectedCount(usize),
    MemberCount(usize),
    Removed,
}

pub fn verify(faction: &mut Faction, checks: &[Assert]) -> ClusterView {
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    for check in checks {
        match *check {
            Assert::PingingCount(n) => assert_eq!(cluster_view.pinging_peers().len(), n),
            Assert::CollectingCount(n) => assert_eq!(cluster_view.collecting_peers().len(), n),
            Assert::Exited => assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped),
            Assert::NotExited => assert_ne!(cluster_view.peer_state(), PeerState::Bootstrapped),
            Assert::Conclusion(mode) => assert_eq!(
                (cluster_view.peer_state() == PeerState::Bootstrapped)
                    .then_some(Conclusion::Bootstrapped),
                Some(mode)
            ),
            Assert::LocalComplete => assert!(cluster_view.is_pinging_completed()),
            Assert::NotLocalComplete => assert!(!cluster_view.is_pinging_completed()),
            Assert::Quorum(quorum) => assert_eq!(cluster_view.quorum(), quorum),
            Assert::SuspectedCount(n) => assert_eq!(cluster_view.suspected().len(), n),
            Assert::MemberCount(n) => assert_eq!(cluster_view.members().as_slice().len(), n),
            Assert::Removed => assert_eq!(cluster_view.peer_state(), PeerState::Removed),
        }
    }
    cluster_view
}
