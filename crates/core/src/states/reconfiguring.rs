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
use crate::stage::Stage;
use crate::state::State;
use crate::types::Epoch;

use super::bootstrapped::Bootstrapped;
use super::in_flight::InFlight;

// A concluded node with changes in flight, each an addition or a removal: it
// answers as a concluded node does, but for an admission, which no change is
// gone around by, and a request, which a node at its bound takes no more; and
// it carries each change to its commit or its abort, the node then in the
// stage the changes left make (§2).
#[derive(Debug, Clone)]
pub struct Reconfiguring {
    concluded: Bootstrapped,
    in_flight: InFlight,
}

impl Reconfiguring {
    #[must_use]
    pub fn new(concluded: Bootstrapped, in_flight: InFlight) -> Self {
        Self {
            concluded,
            in_flight,
        }
    }

    // A change requested of the node, with its changes in flight (§11, §12).
    #[must_use]
    pub fn requested(&self, command: &Command, config: &Config) -> Option<(Vec<Outcome>, Stage)> {
        self.concluded.requested(command, &self.in_flight, config)
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
        if let Some((outcomes, in_flight)) =
            self.in_flight.answered(&command, &self.concluded, epoch)
        {
            return Some((outcomes, in_flight.stage(self.concluded.clone(), config)));
        }
        let in_flight = self.in_flight.ended_by(&command).map_or_else(
            || self.in_flight.clone(),
            |peer_id| self.in_flight.without(peer_id),
        );
        match command {
            Command::JoinApproved { .. } => None,
            Command::MembershipReceived {
                members,
                required_count,
                epoch: adopted,
            } => {
                let (outcomes, concluded) = self.concluded.changed(
                    vec![Outcome::MembershipAdopted { epoch: adopted }]
                        .into_iter()
                        .chain(self.in_flight.abandoned())
                        .collect(),
                    &Members::new(members),
                    QuorumPolicy::new(required_count),
                );
                Some((outcomes, Stage::Bootstrapped(concluded)))
            }
            _ => self
                .concluded
                .answer(command, config, epoch)
                .map(|(outcomes, concluded)| (outcomes, in_flight.stage(concluded, config))),
        }
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        self.in_flight
            .shown(ClusterViewBuilder::from_view(
                &self.concluded.cluster_view(previous),
            ))
            .build()
    }
}
