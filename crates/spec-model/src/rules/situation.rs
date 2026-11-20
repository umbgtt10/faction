// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;

use crate::command::Command;
use crate::in_flight::InFlight;
use crate::liveness::Liveness;
use crate::node_state::NodeState;
use crate::reconfiguration::Reconfiguration;
use crate::rules::change::Change;
use crate::vocabulary::command_name::CommandName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Situation {
    Always,
    NotAdmitted,
    PeerIsNotMember,
    PeerIsMember,
    PeerIsPinging,
    PeerIsCollecting,
    PeerConcludes,
    NodeConcludes,
    AcknowledgementNotAwaited,
    ChangeInFlight,
    DeclineNotAwaited,
    PeerIsPending,
    PeerIsNotPending,
    CountConcludes,
    NodeIsAlone,
    PeerIsSelf,
    PeerIsSuspected,
    ThresholdReached,
    RestoresQuorum(Change),
    LosesQuorum(Change),
    CompletionRestoresQuorum,
    SuspicionLosesQuorum,
    QuorumCannotSpare,
    CompletionLosesQuorum,
    ClaimIsBehind,
    ProvisionalClaimIsBehind,
    NodeIsProvisional,
    CommitIsHeld,
    MembershipIsHeld,
    MembershipNamesNode,
    MembershipOmitsNode,
    MembershipConcludes,
}

impl Situation {
    #[must_use]
    pub fn holds(self, command: &Command, state: &NodeState, admitted: &[CommandName]) -> bool {
        let peer_id = command.peer_id.as_ref();
        match self {
            Situation::Always => true,
            Situation::NotAdmitted => !admitted.contains(&command.name),
            Situation::PeerIsNotMember => !Self::lists(&state.members, peer_id),
            Situation::PeerIsMember => Self::lists(&state.members, peer_id),
            Situation::PeerIsPinging => Self::lists(&state.pinging_peers, peer_id),
            Situation::PeerIsCollecting => Self::lists(&state.collecting_peers, peer_id),
            Situation::PeerConcludes => {
                Self::reaches_required_count(&Change::PeerCollects.applied(command, state))
            }
            Situation::NodeConcludes => {
                Self::reaches_required_count(&Change::NodeCollects.applied(command, state))
            }
            Situation::AcknowledgementNotAwaited => !state.reconfigurations.iter().any(
                |change| matches!(change, Reconfiguration::AdditionRequested(pending) if Some(pending) == peer_id),
            ),
            Situation::ChangeInFlight => state
                .reconfigurations
                .iter()
                .any(|change| Self::names(change, peer_id)),
            Situation::DeclineNotAwaited => !state.reconfigurations.iter().any(|change| {
                matches!(
                    change,
                    Reconfiguration::AdditionAcknowledged(proposed)
                        | Reconfiguration::RemovalRequested(proposed, _) if Some(proposed) == peer_id
                )
            }),
            Situation::PeerIsPending => Self::is_pending(command, &state.reconfigurations),
            Situation::PeerIsNotPending => !Self::is_pending(command, &state.reconfigurations),
            Situation::CountConcludes => {
                Self::reaches_required_count(&Change::CountSet.applied(command, state))
            }
            Situation::NodeIsAlone => Liveness::next_to_probe(state).is_none(),
            Situation::PeerIsSelf => peer_id == Some(&state.peer_id),
            Situation::PeerIsSuspected => Self::lists(&state.suspected, peer_id),
            Situation::ThresholdReached => Self::reaches_threshold(command, state),
            Situation::RestoresQuorum(change) => {
                !Liveness::holds_quorum(state)
                    && Liveness::holds_quorum(&change.applied(command, state))
            }
            Situation::LosesQuorum(change) => {
                Liveness::holds_quorum(state)
                    && !Liveness::holds_quorum(&change.applied(command, state))
            }
            Situation::CompletionRestoresQuorum => {
                Self::is_pending(command, &state.reconfigurations)
                    && Situation::RestoresQuorum(Change::AdditionCompletes)
                        .holds(command, state, admitted)
            }
            Situation::SuspicionLosesQuorum => {
                Self::reaches_threshold(command, state)
                    && Situation::LosesQuorum(Change::PeerSuspected).holds(command, state, admitted)
            }
            Situation::QuorumCannotSpare => {
                let leaves = command.leaves.or_else(|| InFlight::leaves(state, peer_id));
                !Self::lists(&state.suspected, peer_id)
                    && !Liveness::holds_quorum(&NodeState {
                        required_count: leaves.unwrap_or(state.required_count),
                        ..Self::without_removals(&Change::PeerRemoved.applied(command, state))
                    })
            }
            Situation::CompletionLosesQuorum => {
                Self::is_pending(command, &state.reconfigurations)
                    && Situation::LosesQuorum(Change::RemovalCompletes)
                        .holds(command, state, admitted)
            }
            Situation::ClaimIsBehind => command.epoch < state.epoch,
            Situation::ProvisionalClaimIsBehind => state.provisional && command.epoch < state.epoch,
            Situation::NodeIsProvisional => state.provisional,
            Situation::CommitIsHeld => (1..=state.epoch).contains(&command.epoch),
            Situation::MembershipIsHeld => command.epoch <= state.epoch,
            Situation::MembershipNamesNode => command.members.contains(&state.peer_id),
            Situation::MembershipOmitsNode => !command.members.contains(&state.peer_id),
            Situation::MembershipConcludes => {
                Self::reaches_required_count(&Change::MembershipAdopted.applied(command, state))
            }
        }
    }

    // Whether the change the command commits or aborts is in flight: a
    // removal's of `peer_id` for a removal's command, an addition's otherwise.
    fn is_pending(command: &Command, reconfigurations: &[Reconfiguration]) -> bool {
        let peer_id = command.peer_id.as_ref();
        let removal = matches!(
            command.name,
            CommandName::RemovalCommitted | CommandName::RemovalAborted
        );
        reconfigurations.iter().any(|change| match change {
            Reconfiguration::RemovalRequested(pending, _) => removal && Some(pending) == peer_id,
            Reconfiguration::AdditionRequested(pending)
            | Reconfiguration::AdditionAcknowledged(pending) => {
                !removal && Some(pending) == peer_id
            }
        })
    }

    fn lists(peers: &[String], peer_id: Option<&String>) -> bool {
        peer_id.is_some_and(|peer_id| peers.contains(peer_id))
    }

    // Whether `change` names `peer_id`.
    fn names(change: &Reconfiguration, peer_id: Option<&String>) -> bool {
        match change {
            Reconfiguration::AdditionRequested(named)
            | Reconfiguration::AdditionAcknowledged(named)
            | Reconfiguration::RemovalRequested(named, _) => Some(named) == peer_id,
        }
    }

    fn reaches_required_count(state: &NodeState) -> bool {
        u64::try_from(state.collecting_peers.len()).is_ok_and(|count| count >= state.required_count)
    }

    fn reaches_threshold(command: &Command, state: &NodeState) -> bool {
        command.peer_id.as_ref().is_some_and(|peer_id| {
            Liveness::misses(state, peer_id) + 1 >= state.suspicion_threshold
        })
    }

    // The node with every member whose removal is in flight no longer one: the
    // removals in flight counted together (§12).
    fn without_removals(state: &NodeState) -> NodeState {
        NodeState {
            members: state
                .members
                .iter()
                .filter(|member| {
                    !state.reconfigurations.iter().any(|change| {
                        matches!(change, Reconfiguration::RemovalRequested(named, _) if named == *member)
                    })
                })
                .cloned()
                .collect(),
            ..state.clone()
        }
    }
}
