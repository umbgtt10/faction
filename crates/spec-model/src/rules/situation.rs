// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;

use crate::addition::Addition;
use crate::command::Command;
use crate::liveness::Liveness;
use crate::node_state::NodeState;
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
            Situation::AcknowledgementNotAwaited => {
                !matches!(&state.addition, Addition::Requested(pending) if Some(pending) == peer_id)
            }
            Situation::PeerIsPending => Self::is_pending(&state.addition, peer_id),
            Situation::PeerIsNotPending => !Self::is_pending(&state.addition, peer_id),
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
                Self::is_pending(&state.addition, peer_id)
                    && Situation::RestoresQuorum(Change::AdditionCompletes)
                        .holds(command, state, admitted)
            }
            Situation::SuspicionLosesQuorum => {
                Self::reaches_threshold(command, state)
                    && Situation::LosesQuorum(Change::PeerSuspected).holds(command, state, admitted)
            }
        }
    }

    fn is_pending(addition: &Addition, peer_id: Option<&String>) -> bool {
        matches!(
            addition,
            Addition::Requested(pending) | Addition::Acknowledged(pending) if Some(pending) == peer_id
        )
    }

    fn lists(peers: &[String], peer_id: Option<&String>) -> bool {
        peer_id.is_some_and(|peer_id| peers.contains(peer_id))
    }

    fn reaches_required_count(state: &NodeState) -> bool {
        u64::try_from(state.collecting_peers.len()).is_ok_and(|count| count >= state.required_count)
    }

    fn reaches_threshold(command: &Command, state: &NodeState) -> bool {
        command.peer_id.as_ref().is_some_and(|peer_id| {
            Liveness::misses(state, peer_id) + 1 >= state.suspicion_threshold
        })
    }
}
