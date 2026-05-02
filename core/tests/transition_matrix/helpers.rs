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
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::Freshness;
use faction::PeerId;

pub const THRESHOLD: usize = 5;
pub const MAX_DELAY: Freshness = 2;
pub const MARKER: Freshness = 10;
pub const TIMELY: Freshness = 10;
pub const DELAYED: Freshness = 8;
pub const STALE: Freshness = 7;

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
    let mut m = Faction::new(
        Config::new(
            0,
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(THRESHOLD),
            FreshnessPolicy::new(MAX_DELAY),
        ),
        Box::new(NoOpObserver),
    );
    if !matches!(init, Init::Initial) {
        let _ = m.process(Command::ParticipationObserved {
            peer_id: 99,
            freshness: MARKER,
            current_marker: MARKER,
        });
    }
    match init {
        Init::Initial => {}
        Init::Fresh => {}
        Init::PingingPeer1Confirmed => {
            let _ = m.process(Command::ParticipationObserved {
                peer_id: 1,
                freshness: TIMELY,
                current_marker: MARKER,
            });
        }
        Init::PingingP2Threshold => {
            for peer in 0..5 {
                let _ = m.process(Command::ReadyObserved {
                    peer_id: peer,
                    freshness: TIMELY,
                    current_marker: MARKER,
                });
            }
        }
        Init::CollectingNoReadiness => {
            let _ = m.process(Command::LocalParticipationCompleted);
        }
        Init::CollectingPeer1Confirmed => {
            let _ = m.process(Command::LocalParticipationCompleted);
            let _ = m.process(Command::ReadyObserved {
                peer_id: 1,
                freshness: TIMELY,
                current_marker: MARKER,
            });
        }
        Init::CollectingAlmostQuorum => {
            let _ = m.process(Command::LocalParticipationCompleted);
            for peer in 1..4 {
                let _ = m.process(Command::ReadyObserved {
                    peer_id: peer,
                    freshness: TIMELY,
                    current_marker: MARKER,
                });
            }
        }
        Init::Bootstrapped => {
            let _ = m.process(Command::LocalParticipationCompleted);
            for peer in 1..5 {
                let _ = m.process(Command::ReadyObserved {
                    peer_id: peer,
                    freshness: TIMELY,
                    current_marker: MARKER,
                });
            }
        }
        Init::TimedOut => {
            let _ = m.process(Command::DeadlineExpired);
        }
    }
    m
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

pub fn verify(m: &mut Faction, checks: &[Assert]) {
    let s = match m.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    for check in checks {
        match *check {
            Assert::PingingCount(n) => assert_eq!(s.pinging_peers().len(), n),
            Assert::CollectingCount(n) => assert_eq!(s.collecting_peers().len(), n),
            Assert::Exited => assert!(s.is_exited()),
            Assert::NotExited => assert!(!s.is_exited()),
            Assert::Conclusion(mode) => assert_eq!(s.exit_mode(), Some(mode)),
            Assert::LocalComplete => assert!(s.is_pinging_completed()),
            Assert::NotLocalComplete => assert!(!s.is_pinging_completed()),
        }
    }
}

pub fn participation(peer_id: PeerId, freshness: Freshness) -> Command {
    Command::ParticipationObserved {
        peer_id,
        freshness,
        current_marker: MARKER,
    }
}

pub fn ready(peer_id: PeerId, freshness: Freshness) -> Command {
    Command::ReadyObserved {
        peer_id,
        freshness,
        current_marker: MARKER,
    }
}
