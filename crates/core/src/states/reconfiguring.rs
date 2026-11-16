// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::config::Config;
use crate::members::Members;
use crate::outcome::Outcome;
use crate::quorum_policy::QuorumPolicy;
use crate::reconfiguration::Reconfiguration;
use crate::stage::Stage;
use crate::state::State;
use crate::types::Epoch;
use crate::types::PeerId;

use super::bootstrapped::Bootstrapped;
use super::epoch_rule::EpochRule;

// A concluded node with a change in flight, an addition or a removal: it
// answers as a concluded node does, but for an admission, which no change is
// gone around by, and it carries the change to its commit or its abort.
#[derive(Debug, Clone)]
pub struct Reconfiguring {
    concluded: Bootstrapped,
    in_flight: Reconfiguration,
}

impl Reconfiguring {
    #[must_use]
    pub fn new(concluded: Bootstrapped, in_flight: Reconfiguration) -> Self {
        Self {
            concluded,
            in_flight,
        }
    }

    // The awaited acknowledgement answered: refused from behind, proposed
    // otherwise.
    fn acknowledged(&self, peer_id: PeerId, claimed: Epoch, epoch: Epoch) -> (Vec<Outcome>, Stage) {
        match EpochRule::new(epoch).refused(peer_id, claimed) {
            Some(refused) => (refused, Stage::Reconfiguring(self.clone())),
            None => (
                vec![Outcome::ProposeAddition { peer_id }],
                Stage::Reconfiguring(Self {
                    in_flight: Reconfiguration::AdditionAcknowledged { peer_id },
                    ..self.clone()
                }),
            ),
        }
    }

    // Whether an abort or a commit names the change in flight: its peer's
    // addition, requested or acknowledged, or its peer's removal.
    fn names_in_flight(&self, command: &Command) -> bool {
        match (command, self.in_flight) {
            (
                Command::AdditionAborted { peer_id } | Command::AdditionCommitted { peer_id, .. },
                Reconfiguration::AdditionRequested { peer_id: pending }
                | Reconfiguration::AdditionAcknowledged { peer_id: pending },
            )
            | (
                Command::RemovalAborted { peer_id } | Command::RemovalCommitted { peer_id, .. },
                Reconfiguration::RemovalRequested { peer_id: pending },
            ) => *peer_id == pending,
            _ => false,
        }
    }
}

impl State for Reconfiguring {
    fn step(
        &self,
        command: Command,
        config: &Config,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome>, Stage)> {
        if let Some(departure) = self.concluded.departure(&command, config.peer_id()) {
            return Some(departure);
        }
        match command {
            Command::JoinApproved { .. } => None,
            Command::MembershipReceived {
                members,
                required_count,
                epoch: adopted,
            } => {
                let (outcomes, concluded) = self.concluded.changed(
                    vec![
                        Outcome::MembershipAdopted { epoch: adopted },
                        Outcome::ChangeAbandoned {
                            reconfiguration: self.in_flight,
                        },
                    ],
                    &Members::new(members),
                    QuorumPolicy::new(required_count),
                );
                Some((outcomes, Stage::Bootstrapped(concluded)))
            }
            Command::MembershipAcknowledged {
                peer_id,
                epoch: claimed,
            } if self.in_flight == (Reconfiguration::AdditionRequested { peer_id }) => {
                Some(self.acknowledged(peer_id, claimed, epoch))
            }
            Command::MembershipAcknowledged { peer_id, .. } => Some((
                vec![Outcome::AcknowledgementIgnored { peer_id }],
                Stage::Reconfiguring(self.clone()),
            )),
            Command::AdditionAborted { peer_id } | Command::RemovalAborted { peer_id }
                if !self.names_in_flight(&command) =>
            {
                Some((
                    vec![Outcome::AbortIgnored { peer_id }],
                    Stage::Reconfiguring(self.clone()),
                ))
            }
            Command::AdditionAborted { peer_id } => Some((
                vec![Outcome::AdditionWithdrawn { peer_id }],
                Stage::Bootstrapped(self.concluded.clone()),
            )),
            Command::RemovalAborted { peer_id } => Some((
                vec![Outcome::RemovalWithdrawn { peer_id }],
                Stage::Bootstrapped(self.concluded.clone()),
            )),
            Command::AdditionCommitted { .. } | Command::RemovalCommitted { .. }
                if self.names_in_flight(&command) =>
            {
                self.concluded
                    .answer(command, config, epoch)
                    .map(|(outcomes, concluded)| (outcomes, Stage::Bootstrapped(concluded)))
            }
            _ => self
                .concluded
                .answer(command, config, epoch)
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
        ClusterViewBuilder::from_view(&self.concluded.cluster_view(previous))
            .with_reconfiguration(self.in_flight)
            .build()
    }
}
