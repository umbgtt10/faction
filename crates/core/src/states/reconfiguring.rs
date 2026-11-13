// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::addition::Addition;
use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::stage::Stage;
use crate::state::State;
use crate::types::PeerId;

use super::bootstrapped::Bootstrapped;

// A concluded node with an addition in flight: it answers as a concluded node
// does, but for an admission, which no addition is gone around by, and it
// carries the addition to its commit or its abort.
#[derive(Debug, Clone)]
pub struct Reconfiguring {
    concluded: Bootstrapped,
    pending: PeerId,
    acknowledged: bool,
}

impl Reconfiguring {
    #[must_use]
    pub fn new(concluded: Bootstrapped, pending: PeerId) -> Self {
        Self {
            concluded,
            pending,
            acknowledged: false,
        }
    }
}

impl State for Reconfiguring {
    fn step(&self, command: Command, config: &Config) -> Option<(Vec<Outcome>, Stage)> {
        match command {
            Command::JoinApproved { .. } => None,
            Command::MembershipAcknowledged { peer_id } => {
                let awaited = peer_id == self.pending && !self.acknowledged;
                let outcome = if awaited {
                    Outcome::ProposeAddition { peer_id }
                } else {
                    Outcome::AcknowledgementIgnored { peer_id }
                };
                Some((
                    vec![outcome],
                    Stage::Reconfiguring(Self {
                        acknowledged: self.acknowledged || awaited,
                        ..self.clone()
                    }),
                ))
            }
            Command::AdditionAborted { peer_id } if peer_id == self.pending => Some((
                vec![Outcome::AdditionWithdrawn { peer_id }],
                Stage::Bootstrapped(self.concluded.clone()),
            )),
            Command::AdditionAborted { peer_id } => Some((
                vec![Outcome::AbortIgnored { peer_id }],
                Stage::Reconfiguring(self.clone()),
            )),
            Command::AdditionCommitted { peer_id } if peer_id == self.pending => self
                .concluded
                .answer(command, config)
                .map(|(outcomes, concluded)| (outcomes, Stage::Bootstrapped(concluded))),
            _ => self
                .concluded
                .answer(command, config)
                .map(|(outcomes, concluded)| {
                    (
                        outcomes,
                        Stage::Reconfiguring(Self {
                            concluded,
                            ..self.clone()
                        }),
                    )
                }),
        }
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        let addition = if self.acknowledged {
            Addition::Acknowledged {
                peer_id: self.pending,
            }
        } else {
            Addition::Requested {
                peer_id: self.pending,
            }
        };
        ClusterViewBuilder::from_view(&self.concluded.cluster_view(previous))
            .with_addition(addition)
            .build()
    }
}
