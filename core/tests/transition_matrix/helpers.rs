// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::PeerId;

pub const THRESHOLD: usize = 5;

#[derive(Debug, Clone, Copy)]
pub enum Init {
    Initial,
    Fresh,
    PingingPeer1Confirmed,
    PingingP2Threshold,
    CollectingNoReadiness,
    CollectingPeer1Confirmed,
    CollectingAlmostQuorum,
    Bootstrapped,
    TimedOut,
}

pub fn build(init: Init) -> Faction {
    let mut faction = Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(THRESHOLD)),
        Box::new(NoOpObserver),
    );
    if !matches!(init, Init::Initial) {
        let _ = faction.process(Command::ParticipationObserved { peer_id: 99 });
    }
    match init {
        Init::Initial => {}
        Init::Fresh => {}
        Init::PingingPeer1Confirmed => {
            let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
        }
        Init::PingingP2Threshold => {
            for peer in 0..5 {
                let _ = faction.process(Command::ReadyObserved { peer_id: peer });
            }
        }
        Init::CollectingNoReadiness => {
            let _ = faction.process(Command::LocalParticipationCompleted);
        }
        Init::CollectingPeer1Confirmed => {
            let _ = faction.process(Command::LocalParticipationCompleted);
            let _ = faction.process(Command::ReadyObserved { peer_id: 1 });
        }
        Init::CollectingAlmostQuorum => {
            let _ = faction.process(Command::LocalParticipationCompleted);
            for peer in 1..4 {
                let _ = faction.process(Command::ReadyObserved { peer_id: peer });
            }
        }
        Init::Bootstrapped => {
            let _ = faction.process(Command::LocalParticipationCompleted);
            for peer in 1..5 {
                let _ = faction.process(Command::ReadyObserved { peer_id: peer });
            }
        }
        Init::TimedOut => {
            let _ = faction.process(Command::DeadlineExpired);
        }
    }
    faction
}

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

pub fn verify(faction: &mut Faction, checks: &[Assert]) {
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
}

pub fn participation(peer_id: PeerId) -> Command {
    Command::ParticipationObserved { peer_id }
}

pub fn ready(peer_id: PeerId) -> Command {
    Command::ReadyObserved { peer_id }
}
