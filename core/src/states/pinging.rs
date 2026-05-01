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
use crate::PeerId;

use super::bootstrapped::Bootstrapped;
use super::collecting::Collecting;
use super::compute_output::ObservedKind;
use super::compute_output::ObservedOutput;
use super::confirmed_set::ConfirmedSet;
use super::timed_out::TimedOut;

#[derive(Default)]
pub struct Pinging {
    pinging_count: ConfirmedSet,
    collecting_count: ConfirmedSet,
}

impl Pinging {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn non_member_peer(command: &Command, config: &Config) -> Option<PeerId> {
        match command {
            Command::ParticipationObserved { peer_id, .. }
            | Command::ReadyObserved { peer_id, .. }
                if !config.is_member(*peer_id) =>
            {
                Some(*peer_id)
            }
            _ => None,
        }
    }
}

impl State for Pinging {
    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        previous
            .clone()
            .with_peer_state(PeerState::Pinging)
            .with_pinging_peers(self.pinging_count.confirmed_peers().to_vec())
            .with_collecting_peers(self.collecting_count.confirmed_peers().to_vec())
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        let pinging_count = self.pinging_count.clone();
        let collecting_count = self.collecting_count.clone();

        if let Some(peer_id) = Self::non_member_peer(&command, config) {
            return (
                vec![Outcome::NonMemberIgnored { peer_id }],
                Box::new(Self {
                    pinging_count,
                    collecting_count,
                }),
            );
        }

        match command {
            Command::ParticipationObserved {
                peer_id,
                freshness,
                current_marker,
            } => {
                let classification = config
                    .freshness_policy()
                    .classify(current_marker, freshness);
                let is_dup = pinging_count.is_confirmed(peer_id);

                let output = ObservedOutput::new(ObservedKind::Participation, peer_id)
                    .compute_output(classification, is_dup);
                let (new_pinging_count, _) = pinging_count.try_confirm(peer_id, classification);

                (
                    vec![output],
                    Box::new(Self {
                        pinging_count: new_pinging_count,
                        collecting_count,
                    }),
                )
            }

            Command::ReadyObserved {
                peer_id,
                freshness,
                current_marker,
            } => {
                let classification = config
                    .freshness_policy()
                    .classify(current_marker, freshness);
                let is_dup = collecting_count.is_confirmed(peer_id);

                let output = ObservedOutput::new(ObservedKind::Ready, peer_id)
                    .compute_output(classification, is_dup);
                let (new_collecting_count, _) =
                    collecting_count.try_confirm(peer_id, classification);

                (
                    vec![output],
                    Box::new(Self {
                        pinging_count,
                        collecting_count: new_collecting_count,
                    }),
                )
            }

            Command::LocalParticipationCompleted => {
                let (new_collecting_count, _) = collecting_count.confirm(config.peer_id());

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
