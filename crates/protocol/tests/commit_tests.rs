// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction_protocol::commit::Commit;

#[test]
fn command_of_each_commit_is_its_command_at_its_epoch() {
    // Arrange
    let commits: [Commit; 3] = [
        Commit::Addition {
            peer_id: 4,
            epoch: 1,
        },
        Commit::Removal {
            peer_id: 2,
            epoch: 2,
        },
        Commit::RequiredCount {
            required_count: 3,
            epoch: 3,
        },
    ];

    // Act
    let commands = commits.map(|commit| commit.command());

    // Assert
    assert_eq!(
        commands,
        [
            Command::AdditionCommitted {
                peer_id: 4,
                epoch: 1
            },
            Command::RemovalCommitted {
                peer_id: 2,
                epoch: 2
            },
            Command::RequiredCountChanged {
                required_count: 3,
                epoch: 3
            },
        ]
    );
}
