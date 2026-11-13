// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use super::builder::{build, Init};
use crate::support::matrix_table::MatrixTable;
use crate::support::spec_files::SpecFiles;
use faction::command::Command;
use faction::process_result::ProcessResult;
use rstest::rstest;

#[rstest]
#[case(Init::Initial, "Initial")]
#[case(Init::Fresh, "Pinging")]
#[case(Init::CollectingNoReadiness, "Collecting")]
#[case(Init::Bootstrapped, "Bootstrapped")]
#[case(Init::Reconfiguring, "Reconfiguring")]
#[case(Init::Removing, "Reconfiguring")]
#[case(Init::Removed, "Removed")]
fn admissible_set_equals_accepted_commands_plus_probe(#[case] init: Init, #[case] stage: &str) {
    let candidates = [
        Command::ParticipationObserved { peer_id: 0 },
        Command::ReadyObserved { peer_id: 0 },
        Command::LocalParticipationCompleted,
        Command::DeadlineExpired,
        Command::JoinRequested { peer_id: 0 },
        Command::JoinApproved { peer_id: 0 },
        Command::JoinRejected { peer_id: 0 },
        Command::AdditionRequested { peer_id: 0 },
        Command::MembershipAcknowledged { peer_id: 0 },
        Command::AdditionCommitted { peer_id: 0 },
        Command::AdditionAborted { peer_id: 0 },
        Command::RemovalRequested { peer_id: 0 },
        Command::RemovalCommitted { peer_id: 0 },
        Command::RemovalAborted { peer_id: 0 },
        Command::RequiredCountChanged { required_count: 0 },
    ];

    // Arrange — the admissible set the state advertises via Probe
    let admissible = match build(init).process(Command::Probe) {
        ProcessResult::Probed { admissible, .. } => admissible,
        _ => unreachable!(),
    };

    // Act & Assert — the stage admits what the specification's table says it does
    assert_eq!(
        admissible,
        MatrixTable::new(SpecFiles::matrix()).admits(stage),
        "{init:?}: admissible disagrees with the table's {stage}"
    );

    // Act & Assert — Probe is always admissible
    assert!(
        admissible.contains(&Command::Probe),
        "{init:?}: Probe not admissible"
    );

    // Act & Assert — every other command is admissible if it is accepted
    for candidate in &candidates {
        let in_admissible = admissible.contains(candidate);
        let accepted = !matches!(
            build(init).process(*candidate),
            ProcessResult::Rejected { .. }
        );
        assert_eq!(
            in_admissible, accepted,
            "{init:?}: admissible disagrees with accept for {candidate:?}"
        );
    }
}
