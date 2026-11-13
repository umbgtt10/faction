// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::rules::change::Change;
use crate::rules::rule::Rule;
use crate::rules::situation::Situation;

const RULES: [(&str, Rule); 56] = [
    (
        "probed",
        Rule {
            situation: Situation::Always,
            change: Change::Nothing,
        },
    ),
    (
        "not-admitted",
        Rule {
            situation: Situation::NotAdmitted,
            change: Change::Nothing,
        },
    ),
    (
        "non-member-before-completion",
        Rule {
            situation: Situation::PeerIsNotMember,
            change: Change::Nothing,
        },
    ),
    (
        "non-member",
        Rule {
            situation: Situation::PeerIsNotMember,
            change: Change::Nothing,
        },
    ),
    (
        "participation-duplicate",
        Rule {
            situation: Situation::PeerIsPinging,
            change: Change::Nothing,
        },
    ),
    (
        "participation-accepted",
        Rule {
            situation: Situation::Always,
            change: Change::PeerPings,
        },
    ),
    (
        "rejoin-acknowledged",
        Rule {
            situation: Situation::Always,
            change: Change::Nothing,
        },
    ),
    (
        "ready-duplicate",
        Rule {
            situation: Situation::PeerIsCollecting,
            change: Change::Nothing,
        },
    ),
    (
        "ready-before-completion",
        Rule {
            situation: Situation::Always,
            change: Change::PeerCollects,
        },
    ),
    (
        "ready-concluded",
        Rule {
            situation: Situation::PeerConcludes,
            change: Change::PeerCollects,
        },
    ),
    (
        "ready-accepted",
        Rule {
            situation: Situation::Always,
            change: Change::PeerCollects,
        },
    ),
    (
        "local-concluded",
        Rule {
            situation: Situation::NodeConcludes,
            change: Change::NodeCollects,
        },
    ),
    (
        "local-completed",
        Rule {
            situation: Situation::Always,
            change: Change::NodeCollects,
        },
    ),
    (
        "deadline-missed",
        Rule {
            situation: Situation::Always,
            change: Change::MissesDeadline,
        },
    ),
    (
        "join-requested",
        Rule {
            situation: Situation::Always,
            change: Change::Nothing,
        },
    ),
    (
        "join-duplicate-member",
        Rule {
            situation: Situation::PeerIsMember,
            change: Change::Nothing,
        },
    ),
    (
        "join-restores-quorum",
        Rule {
            situation: Situation::RestoresQuorum(Change::PeerJoins),
            change: Change::PeerJoins,
        },
    ),
    (
        "join-admitted",
        Rule {
            situation: Situation::Always,
            change: Change::PeerJoins,
        },
    ),
    (
        "join-denied",
        Rule {
            situation: Situation::Always,
            change: Change::Nothing,
        },
    ),
    (
        "addition-member",
        Rule {
            situation: Situation::PeerIsMember,
            change: Change::Nothing,
        },
    ),
    (
        "addition-requested",
        Rule {
            situation: Situation::Always,
            change: Change::AdditionPends,
        },
    ),
    (
        "acknowledgement-ignored",
        Rule {
            situation: Situation::AcknowledgementNotAwaited,
            change: Change::Nothing,
        },
    ),
    (
        "acknowledgement-accepted",
        Rule {
            situation: Situation::Always,
            change: Change::AdditionAcknowledged,
        },
    ),
    (
        "abort-ignored",
        Rule {
            situation: Situation::PeerIsNotPending,
            change: Change::Nothing,
        },
    ),
    (
        "addition-aborted",
        Rule {
            situation: Situation::Always,
            change: Change::AdditionWithdrawn,
        },
    ),
    (
        "removal-aborted",
        Rule {
            situation: Situation::Always,
            change: Change::RemovalWithdrawn,
        },
    ),
    (
        "commit-member",
        Rule {
            situation: Situation::PeerIsMember,
            change: Change::Nothing,
        },
    ),
    (
        "completion-restores-quorum",
        Rule {
            situation: Situation::CompletionRestoresQuorum,
            change: Change::AdditionCompletes,
        },
    ),
    (
        "addition-completed",
        Rule {
            situation: Situation::PeerIsPending,
            change: Change::AdditionCompletes,
        },
    ),
    (
        "commit-restores-quorum",
        Rule {
            situation: Situation::RestoresQuorum(Change::PeerJoins),
            change: Change::PeerJoins,
        },
    ),
    (
        "addition-committed",
        Rule {
            situation: Situation::Always,
            change: Change::PeerJoins,
        },
    ),
    (
        "removal-non-member",
        Rule {
            situation: Situation::PeerIsNotMember,
            change: Change::Nothing,
        },
    ),
    (
        "removal-refused",
        Rule {
            situation: Situation::QuorumCannotSpare,
            change: Change::Nothing,
        },
    ),
    (
        "removal-requested",
        Rule {
            situation: Situation::Always,
            change: Change::RemovalPends,
        },
    ),
    (
        "commit-non-member",
        Rule {
            situation: Situation::PeerIsNotMember,
            change: Change::Nothing,
        },
    ),
    (
        "node-removed",
        Rule {
            situation: Situation::PeerIsSelf,
            change: Change::NodeRemoved,
        },
    ),
    (
        "removal-completion-loses-quorum",
        Rule {
            situation: Situation::CompletionLosesQuorum,
            change: Change::RemovalCompletes,
        },
    ),
    (
        "removal-completed",
        Rule {
            situation: Situation::PeerIsPending,
            change: Change::RemovalCompletes,
        },
    ),
    (
        "removal-loses-quorum",
        Rule {
            situation: Situation::LosesQuorum(Change::PeerRemoved),
            change: Change::PeerRemoved,
        },
    ),
    (
        "removal-committed",
        Rule {
            situation: Situation::Always,
            change: Change::PeerRemoved,
        },
    ),
    (
        "count-concluded",
        Rule {
            situation: Situation::CountConcludes,
            change: Change::CountSet,
        },
    ),
    (
        "count-loses-quorum",
        Rule {
            situation: Situation::LosesQuorum(Change::CountSet),
            change: Change::CountSet,
        },
    ),
    (
        "count-restores-quorum",
        Rule {
            situation: Situation::RestoresQuorum(Change::CountSet),
            change: Change::CountSet,
        },
    ),
    (
        "count-set",
        Rule {
            situation: Situation::Always,
            change: Change::CountSet,
        },
    ),
    (
        "probe-alone",
        Rule {
            situation: Situation::NodeIsAlone,
            change: Change::Nothing,
        },
    ),
    (
        "probe-due",
        Rule {
            situation: Situation::Always,
            change: Change::ProbeMoves,
        },
    ),
    (
        "liveness-self",
        Rule {
            situation: Situation::PeerIsSelf,
            change: Change::Nothing,
        },
    ),
    (
        "liveness-non-member",
        Rule {
            situation: Situation::PeerIsNotMember,
            change: Change::Nothing,
        },
    ),
    (
        "revival-restores-quorum",
        Rule {
            situation: Situation::RestoresQuorum(Change::PeerRevived),
            change: Change::PeerRevived,
        },
    ),
    (
        "peer-revived",
        Rule {
            situation: Situation::PeerIsSuspected,
            change: Change::PeerRevived,
        },
    ),
    (
        "alive-noted",
        Rule {
            situation: Situation::Always,
            change: Change::MissesCleared,
        },
    ),
    (
        "miss-of-a-suspect",
        Rule {
            situation: Situation::PeerIsSuspected,
            change: Change::Nothing,
        },
    ),
    (
        "probe-missed",
        Rule {
            situation: Situation::Always,
            change: Change::Nothing,
        },
    ),
    (
        "suspicion-loses-quorum",
        Rule {
            situation: Situation::SuspicionLosesQuorum,
            change: Change::PeerSuspected,
        },
    ),
    (
        "peer-suspected",
        Rule {
            situation: Situation::ThresholdReached,
            change: Change::PeerSuspected,
        },
    ),
    (
        "miss-counted",
        Rule {
            situation: Situation::Always,
            change: Change::MissCounted,
        },
    ),
];

// What the prose gives each row of the decision table: the situation in which
// it applies and the change it makes. The table gives the rest.
pub struct RuleBook;

impl RuleBook {
    #[must_use]
    pub fn row_ids() -> Vec<&'static str> {
        RULES.iter().map(|(id, _)| *id).collect()
    }

    #[must_use]
    pub fn rule(row_id: &str) -> Rule {
        RULES
            .iter()
            .find(|(id, _)| *id == row_id)
            .map(|(_, rule)| *rule)
            .expect("a row the model knows")
    }
}
