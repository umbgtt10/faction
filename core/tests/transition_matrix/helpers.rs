// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
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
    Phase1Peer1Confirmed,
    Phase1P2Threshold,
    Phase2NoReadiness,
    Phase2Peer1Confirmed,
    Phase2AlmostQuorum,
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
        Init::Phase1Peer1Confirmed => {
            let _ = m.process(Command::ParticipationObserved {
                peer_id: 1,
                freshness: TIMELY,
                current_marker: MARKER,
            });
        }
        Init::Phase1P2Threshold => {
            for peer in 0..5 {
                let _ = m.process(Command::ReadyObserved {
                    peer_id: peer,
                    freshness: TIMELY,
                    current_marker: MARKER,
                });
            }
        }
        Init::Phase2NoReadiness => {
            let _ = m.process(Command::LocalParticipationCompleted);
        }
        Init::Phase2Peer1Confirmed => {
            let _ = m.process(Command::LocalParticipationCompleted);
            let _ = m.process(Command::ReadyObserved {
                peer_id: 1,
                freshness: TIMELY,
                current_marker: MARKER,
            });
        }
        Init::Phase2AlmostQuorum => {
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
    P1Count(usize),
    P2Count(usize),
    Exited,
    NotExited,
    ExitMode(ReadinessExitMode),
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
            Assert::P1Count(n) => assert_eq!(s.phase1_confirmed_count(), n),
            Assert::P2Count(n) => assert_eq!(s.phase2_confirmed_count(), n),
            Assert::Exited => assert!(s.readiness_exited()),
            Assert::NotExited => assert!(!s.readiness_exited()),
            Assert::ExitMode(mode) => assert_eq!(s.exit_mode(), Some(mode)),
            Assert::LocalComplete => assert!(s.local_participation_complete()),
            Assert::NotLocalComplete => assert!(!s.local_participation_complete()),
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
