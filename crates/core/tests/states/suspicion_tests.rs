// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::members::Members;
use faction::miss_count::MissCount;
use faction::outcome::Outcome;
use faction::quorum_policy::QuorumPolicy;
use faction::states::suspicion::Suspicion;

extern crate alloc;

fn members() -> Members {
    Members::new(vec![0, 1, 2])
}

// Member 1 suspected at a threshold of one.
fn suspecting_one() -> Suspicion {
    Suspicion::default().missed_period(1, &members(), 1).1
}

#[test]
fn heard_of_a_member_alive_notes_it_and_forgets_its_misses() {
    // Arrange
    let (_, suspicion) = Suspicion::default().missed_period(2, &members(), 3);

    // Act
    let (outcome, heard) = suspicion.heard(2);

    // Assert
    assert_eq!(outcome, Outcome::AliveNoted { peer_id: 2 });
    assert!(heard.misses().is_empty());
}

#[test]
fn heard_of_a_suspect_revives_it() {
    // Arrange
    let suspicion = suspecting_one();

    // Act
    let (outcome, heard) = suspicion.heard(1);

    // Assert
    assert_eq!(outcome, Outcome::PeerRevived { peer_id: 1 });
    assert!(!heard.is_suspected(1));
}

#[test]
fn holds_quorum_once_a_suspect_takes_the_others_below_the_count_is_false() {
    // Arrange
    let suspicion = suspecting_one();

    // Act
    let holds = suspicion.holds_quorum(&members(), QuorumPolicy::new(3));

    // Assert
    assert!(!holds);
}

#[test]
fn holds_quorum_while_the_members_not_suspected_reach_the_count_is_true() {
    // Arrange
    let suspicion = suspecting_one();

    // Act
    let holds = suspicion.holds_quorum(&members(), QuorumPolicy::new(2));

    // Assert
    assert!(holds);
}

#[test]
fn is_suspected_of_a_member_never_missed_is_false() {
    // Arrange
    let suspicion = suspecting_one();

    // Act
    let suspected = suspicion.is_suspected(2);

    // Assert
    assert!(!suspected);
}

#[test]
fn missed_period_at_the_threshold_suspects_the_member_and_forgets_its_misses() {
    // Arrange
    let (_, suspicion) = Suspicion::default().missed_period(1, &members(), 2);

    // Act
    let (outcome, missed) = suspicion.missed_period(1, &members(), 2);

    // Assert
    assert_eq!(outcome, Outcome::PeerSuspected { peer_id: 1 });
    assert_eq!(missed.suspected(), &[1]);
    assert!(missed.misses().is_empty());
}

#[test]
fn missed_period_below_the_threshold_counts_it() {
    // Arrange
    let suspicion = Suspicion::default();

    // Act
    let (outcome, missed) = suspicion.missed_period(1, &members(), 2);

    // Assert
    assert_eq!(
        outcome,
        Outcome::MissCounted {
            peer_id: 1,
            misses: 1
        }
    );
    assert!(missed.suspected().is_empty());
}

#[test]
fn misses_are_listed_in_the_members_order() {
    // Arrange
    let (_, suspicion) = Suspicion::default().missed_period(2, &members(), 3);
    let (_, suspicion) = suspicion.missed_period(1, &members(), 3);
    let (_, suspicion) = suspicion.missed_period(2, &members(), 3);

    // Act
    let misses = suspicion.misses();

    // Assert
    assert_eq!(
        misses,
        &[
            MissCount {
                peer_id: 1,
                misses: 1
            },
            MissCount {
                peer_id: 2,
                misses: 2
            },
        ]
    );
}

#[test]
fn of_members_forgets_the_suspects_and_the_misses_of_peers_no_longer_members() {
    // Arrange
    let (_, missed) = suspecting_one().missed_period(2, &members(), 3);

    // Act
    let kept = missed.of_members(&Members::new(vec![0, 2]));
    let without_two = missed.of_members(&Members::new(vec![0, 1]));

    // Assert
    assert!(kept.suspected().is_empty());
    assert_eq!(
        kept.misses(),
        &[MissCount {
            peer_id: 2,
            misses: 1
        }]
    );
    assert_eq!(without_two.suspected(), &[1]);
    assert!(without_two.misses().is_empty());
}

#[test]
fn suspected_lists_the_suspects_in_the_order_suspected() {
    // Arrange
    let (_, suspicion) = Suspicion::default().missed_period(2, &members(), 1);
    let (_, suspicion) = suspicion.missed_period(1, &members(), 1);

    // Act
    let suspected = suspicion.suspected();

    // Assert
    assert_eq!(suspected, &[2, 1]);
}
