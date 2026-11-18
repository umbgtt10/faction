// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::instance::instance_id::InstanceId;
use faction::no_op_observer::NoOpObserver;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;

extern crate alloc;

fn fresh() -> Faction {
    Faction::new(
        Config::new(
            InstanceId {
                node: 0,
                incarnation: 1,
            },
            vec![0, 1, 2],
            QuorumPolicy::new(2),
            1,
            0,
            false,
            1,
        ),
        Box::new(NoOpObserver),
    )
}

// A fresh node admits every kind but the deadline and a concluded node's own:
// a command's kind is among those it lists exactly when it accepts the
// command.
#[test]
fn kind_of_a_command_is_admitted_exactly_when_a_fresh_node_accepts_it() {
    // Arrange
    let commands = [
        Command::ParticipationObserved {
            peer_id: 1,
            epoch: 0,
        },
        Command::DeadlineExpired,
        Command::JoinRequested { peer_id: 7 },
        Command::AdditionRequested { peer_id: 7 },
        Command::ProbeDue,
    ];
    let admissible = match fresh().process(Command::Probe).output {
        ProcessResult::Probed { admissible, .. } => admissible,
        other => panic!("expected Probed, got {other:?}"),
    };

    // Act
    let admitted: Vec<bool> = commands
        .iter()
        .map(|command| admissible.contains(&command.kind()))
        .collect();

    // Assert
    let accepted: Vec<bool> = commands
        .iter()
        .map(|command| {
            matches!(
                fresh().process(command.clone()).output,
                ProcessResult::Accepted { .. }
            )
        })
        .collect();
    assert_eq!(admitted, vec![true, false, true, false, false]);
    assert_eq!(admitted, accepted);
}
