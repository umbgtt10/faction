// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::recording_observer::RecordingObserver;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use std::sync::Arc;
use std::sync::Mutex;

extern crate alloc;

fn faction(observer: Box<dyn Observer>) -> Faction {
    Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4)),
        observer,
    )
}

#[test]
fn process_given_a_no_op_observer_answers_as_given_one_that_records() {
    // Arrange
    let mut silent = faction(Box::new(NoOpObserver));
    let mut recorded = faction(Box::new(RecordingObserver::new(Arc::new(Mutex::new(
        Vec::new(),
    )))));
    let commands = vec![
        Command::DeadlineExpired,
        Command::Probe,
        Command::ParticipationObserved { peer_id: 1 },
        Command::LocalParticipationCompleted,
        Command::LocalParticipationCompleted,
        Command::ReadyObserved { peer_id: 1 },
        Command::ReadyObserved { peer_id: 2 },
        Command::ReadyObserved { peer_id: 3 },
    ];

    // Act
    let answers: Vec<(ProcessResult, ProcessResult)> = commands
        .into_iter()
        .map(|command| (silent.process(command), recorded.process(command)))
        .collect();

    // Assert
    for (from_silent, from_recorded) in answers {
        assert_eq!(from_silent, from_recorded);
    }
}
