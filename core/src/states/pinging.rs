// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::exit_mode::ExitMode;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::state::State;

use super::bootstrapped::Bootstrapped;
use super::collecting::Collecting;
use super::compute_output::ObservedKind;
use super::compute_output::ObservedOutput;
use super::confirmed_set::ConfirmedSet;
use super::timed_out::TimedOut;

pub struct Pinging {
    pinging_count: ConfirmedSet,
    collecting_count: ConfirmedSet,
}

impl Pinging {
    #[must_use]
    pub fn new(peer_count: usize) -> Self {
        Self {
            pinging_count: ConfirmedSet::new(peer_count),
            collecting_count: ConfirmedSet::new(peer_count),
        }
    }
}

impl State for Pinging {
    fn cluster_view(&self, previous: &ClusterView, config: &Config) -> ClusterView {
        previous
            .clone()
            .with_peer_state(PeerState::Pinging)
            .with_pinging_peers(self.pinging_count.confirmed_peers(config.peers()))
            .with_collecting_peers(self.collecting_count.confirmed_peers(config.peers()))
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        let pinging_count = self.pinging_count.clone();
        let collecting_count = self.collecting_count.clone();

        match command {
            Command::ParticipationObserved {
                peer_id,
                freshness,
                current_marker,
            } => {
                let index = config.peer_index(peer_id);
                let classification = index.map(|_| {
                    config
                        .freshness_policy()
                        .classify(current_marker, freshness)
                });
                let is_dup = index.is_some_and(|i| pinging_count.is_confirmed(i));

                let output = ObservedOutput::new(ObservedKind::Participation, peer_id)
                    .compute_output(index, classification, is_dup);
                let (phase1, _) = pinging_count.try_confirm(index, is_dup, classification);

                (vec![output], Box::new(Self { pinging_count: phase1, collecting_count }))
            }

            Command::ReadyObserved {
                peer_id,
                freshness,
                current_marker,
            } => {
                let index = config.peer_index(peer_id);
                let classification = index.map(|_| {
                    config
                        .freshness_policy()
                        .classify(current_marker, freshness)
                });
                let is_dup = index.is_some_and(|i| collecting_count.is_confirmed(i));

                let output = ObservedOutput::new(ObservedKind::Ready, peer_id).compute_output(
                    index,
                    classification,
                    is_dup,
                );
                let (new_collecting_count, _) = collecting_count.try_confirm(index, is_dup, classification);

                (vec![output], Box::new(Self { pinging_count, collecting_count: new_collecting_count }))
            }

            Command::LocalParticipationCompleted => {
                let local_index = config
                    .peer_index(config.local_peer_id())
                    .expect("local peer must be in peer set");
                let (new_collecting_count, _) = collecting_count.confirm(local_index);

                let mut outputs = vec![
                    Outcome::LocalParticipationCompleted,
                    Outcome::BroadcastLocalReady,
                ];

                let quorum = new_collecting_count.count() >= config.required_count();
                if quorum {
                    outputs.push(Outcome::ReadyQuorumReached);
                    outputs.push(Outcome::Exited {
                        mode: ExitMode::Bootstrapped,
                    });
                }

                let new_state: Box<dyn State> = if quorum {
                    Box::new(Bootstrapped {
                        pinging_count: pinging_count.count(),
                        collecting_count: new_collecting_count.count(),
                    })
                } else {
                    Box::new(Collecting {
                        collecting_count: new_collecting_count,
                        pinging_count: pinging_count.count(),
                    })
                };
                (outputs, new_state)
            }

            Command::DeadlineExpired => (
                vec![Outcome::Exited {
                    mode: ExitMode::TimedOut,
                }],
                Box::new(TimedOut {
                    pinging_count: pinging_count.count(),
                    collecting_count: collecting_count.count(),
                }),
            ),

            Command::Probe => unreachable!("Probe handled in Faction::process"),
        }
    }
}
