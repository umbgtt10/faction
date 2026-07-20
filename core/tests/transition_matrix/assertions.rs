// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::faction::Faction;
use faction::process_result::ProcessResult;

#[derive(Debug, Clone, Copy)]
pub enum Assert {
    PingingCount(usize),
    CollectingCount(usize),
    Exited,
    NotExited,
    Conclusion(Conclusion),
    LocalComplete,
    NotLocalComplete,
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
            Assert::Exited => assert!(cluster_view.is_concluded()),
            Assert::NotExited => assert!(!cluster_view.is_concluded()),
            Assert::Conclusion(mode) => assert_eq!(cluster_view.conclusion(), Some(mode)),
            Assert::LocalComplete => assert!(cluster_view.is_pinging_completed()),
            Assert::NotLocalComplete => assert!(!cluster_view.is_pinging_completed()),
        }
    }
    cluster_view
}
