// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::quorum_policy::QuorumPolicy;

// The peer whose addition a reconfiguring node has in flight.
pub const NEWCOMER: u64 = 9;
pub const THRESHOLD: usize = 5;

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
        Init::Reconfiguring | Init::ReconfiguringAcknowledged => {
            let _ = faction.process(Command::LocalParticipationCompleted);
            for peer in 1..5 {
                let _ = faction.process(Command::ReadyObserved { peer_id: peer });
            }
            let _ = faction.process(Command::AdditionRequested { peer_id: NEWCOMER });
        }
    }
    if matches!(init, Init::ReconfiguringAcknowledged) {
        let _ = faction.process(Command::MembershipAcknowledged { peer_id: NEWCOMER });
    }
    faction
}

extern crate alloc;

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
    Reconfiguring,
    ReconfiguringAcknowledged,
}
