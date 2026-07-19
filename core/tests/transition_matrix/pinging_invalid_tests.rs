// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::process_result::ProcessResult;
use rstest::rstest;

use super::builder::{build, Init};
use super::helpers::{participation, ready};

#[rstest]
#[case(participation(1))]
#[case(ready(1))]
#[case(Command::LocalParticipationCompleted)]
#[case(Command::DeadlineExpired)]
fn pinging_accepts_every_non_probe_command(#[case] command: Command) {
    // Arrange
    let mut faction = build(Init::Fresh);

    // Act & Assert — Pinging has no invalid transitions
    assert!(matches!(
        faction.process(command),
        ProcessResult::Accepted { .. }
    ));
}
