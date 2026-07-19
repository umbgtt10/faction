// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::process_result::ProcessResult;
use rstest::rstest;

use super::builder::{build, Init};

#[rstest]
#[case(Init::Initial)]
#[case(Init::Fresh)]
#[case(Init::CollectingNoReadiness)]
#[case(Init::Bootstrapped)]
fn admissible_set_equals_accepted_commands_plus_probe(#[case] init: Init) {
    let candidates = [
        Command::ParticipationObserved { peer_id: 0 },
        Command::ReadyObserved { peer_id: 0 },
        Command::LocalParticipationCompleted,
        Command::DeadlineExpired,
    ];

    // Arrange — the admissible set the state advertises via Probe
    let admissible = match build(init).process(Command::Probe) {
        ProcessResult::Probed { admissible, .. } => admissible,
        _ => unreachable!(),
    };

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
