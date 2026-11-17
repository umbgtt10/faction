// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::harness::committed_log::CommittedLog;
use faction_chaos::model::change::Change;
use faction_protocol::commit::Commit;
use faction_system_tests::infra::proposal::Proposal;
use faction_system_tests::membership::Membership;

fn log() -> CommittedLog {
    CommittedLog::new(Membership {
        peers: vec![0, 1, 2],
        required_count: 2,
        epoch: 0,
    })
}

// A log of three commits: 3 added, 1 removed, three required.
fn log_of_three() -> CommittedLog {
    let mut log = log();
    log.commit(Change::Addition { peer_id: 3 });
    log.commit(Change::Removal { peer_id: 1 });
    log.commit(Change::RequiredCount { required_count: 3 });
    log
}

#[test]
fn admits_a_proposal_decided_at_the_log_s_epoch() {
    // Arrange
    let log = log_of_three();

    // Act
    let admitted = [
        log.admits(Proposal::Addition {
            peer_id: 1,
            epoch: 3,
        }),
        log.admits(Proposal::Removal {
            peer_id: 0,
            epoch: 3,
        }),
    ];

    // Assert
    assert_eq!(admitted, [true, true]);
}

#[test]
fn admits_a_removal_unless_another_removal_or_a_count_committed_since() {
    // Arrange
    let log = log_of_three();

    // Act
    let admitted = [
        log.admits(Proposal::Removal {
            peer_id: 0,
            epoch: 0,
        }),
        log.admits(Proposal::Removal {
            peer_id: 0,
            epoch: 2,
        }),
        log.admits(Proposal::Removal {
            peer_id: 0,
            epoch: 1,
        }),
    ];

    // Assert
    assert_eq!(admitted, [false, false, false]);
}

#[test]
fn admits_a_removal_with_only_another_peer_s_addition_committed_since() {
    // Arrange
    let mut log = log();
    log.commit(Change::Addition { peer_id: 3 });

    // Act
    let admitted = [
        log.admits(Proposal::Removal {
            peer_id: 0,
            epoch: 0,
        }),
        log.admits(Proposal::Removal {
            peer_id: 3,
            epoch: 0,
        }),
    ];

    // Assert
    assert_eq!(admitted, [true, false]);
}

#[test]
fn admits_an_addition_unless_a_change_naming_its_peer_committed_since() {
    // Arrange
    let log = log_of_three();

    // Act
    let admitted = [
        log.admits(Proposal::Addition {
            peer_id: 1,
            epoch: 1,
        }),
        log.admits(Proposal::Addition {
            peer_id: 4,
            epoch: 0,
        }),
    ];

    // Assert
    assert_eq!(admitted, [false, true]);
}

#[test]
fn after_an_epoch_is_the_place_of_the_first_commit_past_it() {
    // Arrange
    let log = log_of_three();

    // Act
    let places = [log.after(0), log.after(2), log.after(9)];

    // Assert
    assert_eq!(places, [0, 2, 3]);
}

#[test]
fn commit_numbers_the_changes_from_one_in_order() {
    // Arrange
    let mut log = log();

    // Act
    let first = log.commit(Change::Addition { peer_id: 3 });
    let second = log.commit(Change::RequiredCount { required_count: 1 });

    // Assert
    assert_eq!(
        first,
        Commit::Addition {
            peer_id: 3,
            epoch: 1
        }
    );
    assert_eq!(
        second,
        Commit::RequiredCount {
            required_count: 1,
            epoch: 2
        }
    );
}

#[test]
fn commits_lists_every_commit_in_order() {
    // Arrange
    let log = log_of_three();

    // Act
    let commits = log.commits();

    // Assert
    assert_eq!(
        commits,
        [
            Commit::Addition {
                peer_id: 3,
                epoch: 1
            },
            Commit::Removal {
                peer_id: 1,
                epoch: 2
            },
            Commit::RequiredCount {
                required_count: 3,
                epoch: 3
            },
        ]
    );
}

#[test]
fn fits_a_change_its_membership_can_hold() {
    // Arrange
    let log = log();

    // Act
    let fitting = [
        log.fits(Change::Addition { peer_id: 3 }),
        log.fits(Change::Addition { peer_id: 1 }),
        log.fits(Change::Removal { peer_id: 1 }),
        log.fits(Change::Removal { peer_id: 5 }),
        log.fits(Change::RequiredCount { required_count: 3 }),
        log.fits(Change::RequiredCount { required_count: 4 }),
        log.fits(Change::RequiredCount { required_count: 0 }),
    ];

    // Assert
    assert_eq!(fitting, [true, false, true, false, true, false, false]);
}

#[test]
fn membership_at_a_prefix_holds_only_its_commits() {
    // Arrange
    let log = log_of_three();

    // Act
    let membership = log.membership_at(1);

    // Assert
    assert_eq!(
        membership,
        Membership {
            peers: vec![0, 1, 2, 3],
            required_count: 2,
            epoch: 1,
        }
    );
}

#[test]
fn membership_holds_every_commit_s_change() {
    // Arrange
    let log = log_of_three();

    // Act
    let membership = log.membership();

    // Assert
    assert_eq!(
        membership,
        Membership {
            peers: vec![0, 2, 3],
            required_count: 3,
            epoch: 3,
        }
    );
}

#[test]
fn removed_at_of_a_peer_left_removed_is_the_epoch_of_its_removal() {
    // Arrange
    let log = log_of_three();

    // Act
    let removed = [log.removed_at(1), log.removed_at(0)];

    // Assert
    assert_eq!(removed, [Some(2), None]);
}

#[test]
fn removed_at_of_a_peer_taken_back_is_none() {
    // Arrange
    let mut log = log_of_three();
    log.commit(Change::Addition { peer_id: 1 });

    // Act
    let removed = log.removed_at(1);

    // Assert
    assert_eq!(removed, None);
}
