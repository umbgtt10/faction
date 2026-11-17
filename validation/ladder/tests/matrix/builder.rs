// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::instance::instance_id::InstanceId;
use faction::no_op_observer::NoOpObserver;
use faction::quorum_policy::QuorumPolicy;

// The epoch a node built at an epoch holds: the count's commit took it there.
pub const EPOCH: u64 = 2;
// The peer whose addition a reconfiguring node has in flight.
pub const NEWCOMER: u64 = 9;
// The member a removing node suspects, and whose removal it has in flight.
pub const SUSPECT: u64 = 4;
pub const THRESHOLD: usize = 5;

pub fn build(init: Init) -> Faction {
    let mut faction = Faction::new(
        Config::new(
            InstanceId {
                node: 0,
                incarnation: 1,
            },
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(THRESHOLD),
            1,
            0,
            false,
        ),
        Box::new(NoOpObserver),
    );
    if !matches!(init, Init::Initial | Init::InitialAtEpoch) {
        let _ = faction
            .process(Command::ParticipationObserved {
                peer_id: 99,
                epoch: 0,
            })
            .output;
    }
    match init {
        Init::Initial | Init::InitialAtEpoch => {}
        Init::Fresh => {}
        Init::PingingPeer1Confirmed => {
            let _ = faction
                .process(Command::ParticipationObserved {
                    peer_id: 1,
                    epoch: 0,
                })
                .output;
        }
        Init::PingingP2Threshold => {
            for peer in 0..5 {
                let _ = faction
                    .process(Command::ReadyObserved {
                        peer_id: peer,
                        epoch: 0,
                    })
                    .output;
            }
        }
        Init::CollectingNoReadiness | Init::CollectingAtEpoch => {
            let _ = faction.process(Command::LocalParticipationCompleted).output;
        }
        Init::CollectingPeer1Confirmed => {
            let _ = faction.process(Command::LocalParticipationCompleted).output;
            let _ = faction
                .process(Command::ReadyObserved {
                    peer_id: 1,
                    epoch: 0,
                })
                .output;
        }
        Init::CollectingAlmostQuorum => {
            let _ = faction.process(Command::LocalParticipationCompleted).output;
            for peer in 1..4 {
                let _ = faction
                    .process(Command::ReadyObserved {
                        peer_id: peer,
                        epoch: 0,
                    })
                    .output;
            }
        }
        Init::Bootstrapped | Init::BootstrappedAtEpoch => {
            let _ = faction.process(Command::LocalParticipationCompleted).output;
            for peer in 1..5 {
                let _ = faction
                    .process(Command::ReadyObserved {
                        peer_id: peer,
                        epoch: 0,
                    })
                    .output;
            }
        }
        Init::TimedOut => {
            let _ = faction.process(Command::DeadlineExpired).output;
        }
        Init::Suspecting | Init::Removing | Init::RemovingRevived => {
            let _ = faction.process(Command::LocalParticipationCompleted).output;
            for peer in 1..5 {
                let _ = faction
                    .process(Command::ReadyObserved {
                        peer_id: peer,
                        epoch: 0,
                    })
                    .output;
            }
            let _ = faction
                .process(Command::IndirectProbesMissed { peer_id: SUSPECT })
                .output;
        }
        Init::Removed => {
            let _ = faction.process(Command::LocalParticipationCompleted).output;
            for peer in 1..5 {
                let _ = faction
                    .process(Command::ReadyObserved {
                        peer_id: peer,
                        epoch: 0,
                    })
                    .output;
            }
            let _ = faction
                .process(Command::RemovalCommitted {
                    peer_id: 0,
                    epoch: 0,
                })
                .output;
        }
        Init::Reconfiguring | Init::ReconfiguringAcknowledged | Init::ReconfiguringAtEpoch => {
            let _ = faction.process(Command::LocalParticipationCompleted).output;
            for peer in 1..5 {
                let _ = faction
                    .process(Command::ReadyObserved {
                        peer_id: peer,
                        epoch: 0,
                    })
                    .output;
            }
            let _ = faction
                .process(Command::AdditionRequested { peer_id: NEWCOMER })
                .output;
        }
    }
    if matches!(init, Init::ReconfiguringAcknowledged) {
        let _ = faction
            .process(Command::MembershipAcknowledged {
                peer_id: NEWCOMER,
                epoch: 0,
            })
            .output;
    }
    if matches!(init, Init::Removing | Init::RemovingRevived) {
        let _ = faction
            .process(Command::RemovalRequested { peer_id: SUSPECT })
            .output;
    }
    if matches!(init, Init::RemovingRevived) {
        let _ = faction
            .process(Command::PeerHeard { peer_id: SUSPECT })
            .output;
    }
    if matches!(
        init,
        Init::InitialAtEpoch
            | Init::CollectingAtEpoch
            | Init::BootstrappedAtEpoch
            | Init::ReconfiguringAtEpoch
    ) {
        let _ = faction
            .process(Command::RequiredCountChanged {
                required_count: THRESHOLD,
                epoch: EPOCH,
            })
            .output;
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
    Suspecting,
    Removing,
    RemovingRevived,
    Removed,
    InitialAtEpoch,
    CollectingAtEpoch,
    BootstrappedAtEpoch,
    ReconfiguringAtEpoch,
}
