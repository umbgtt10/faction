// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::quorum_policy::QuorumPolicy;

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
