// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;
use faction::command::Command;
use faction::config::Config;
use faction::members::Members;
use faction::miss_count::MissCount;
use faction::outcome::Outcome;
use faction::quorum_policy::QuorumPolicy;
use faction::states::liveness::Liveness;

fn config(threshold: usize) -> Config {
    Config::new(0, vec![0, 1, 2], QuorumPolicy::new(2), threshold)
}

extern crate alloc;

fn members() -> Members {
    Members::new(vec![0, 1, 2])
}

// The liveness after each command in turn, the outcomes of the last.
fn stepped(commands: &[Command], threshold: usize) -> (Vec<Outcome>, Liveness) {
    commands.iter().fold(
        (Vec::new(), Liveness::default()),
        |(_, liveness), command| {
            liveness
                .step(
                    *command,
                    &members(),
                    QuorumPolicy::new(2),
                    &config(threshold),
                )
                .expect("a command of liveness")
        },
    )
}

#[test]
fn crossing_from_held_to_held_says_nothing() {
    // Arrange & Act
    let crossing = Liveness::crossing(true, true);

    // Assert
    assert_eq!(crossing, None);
}

#[test]
fn crossing_from_held_to_short_is_quorum_lost() {
    // Arrange & Act
    let crossing = Liveness::crossing(true, false);

    // Assert
    assert_eq!(crossing, Some(Outcome::QuorumLost));
}

#[test]
fn crossing_from_short_to_held_is_quorum_restored() {
    // Arrange & Act
    let crossing = Liveness::crossing(false, true);

    // Assert
    assert_eq!(crossing, Some(Outcome::QuorumRestored));
}

#[test]
fn holds_quorum_once_too_many_are_suspected_is_false() {
    // Arrange
    let (_, liveness) = stepped(
        &[
            Command::IndirectProbesMissed { peer_id: 1 },
            Command::IndirectProbesMissed { peer_id: 2 },
        ],
        1,
    );

    // Act
    let holds = liveness.holds_quorum(&members(), QuorumPolicy::new(2));

    // Assert
    assert!(!holds);
}

#[test]
fn holds_quorum_with_every_member_alive_is_true() {
    // Arrange
    let liveness = Liveness::default();

    // Act
    let holds = liveness.holds_quorum(&members(), QuorumPolicy::new(3));

    // Assert
    assert!(holds);
}

#[test]
fn last_probed_after_a_probe_period_is_the_member_probed() {
    // Arrange
    let (_, liveness) = stepped(&[Command::ProbeDue, Command::ProbeDue], 1);

    // Act
    let last_probed = liveness.last_probed();

    // Assert
    assert_eq!(last_probed, Some(2));
}

#[test]
fn misses_below_the_threshold_are_counted_in_the_members_order() {
    // Arrange
    let (_, liveness) = stepped(
        &[
            Command::IndirectProbesMissed { peer_id: 2 },
            Command::IndirectProbesMissed { peer_id: 1 },
        ],
        3,
    );

    // Act
    let misses = liveness.misses();

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
                misses: 1
            },
        ]
    );
}

#[test]
fn of_members_forgets_whom_the_node_no_longer_counts() {
    // Arrange
    let (_, liveness) = stepped(
        &[
            Command::ProbeDue,
            Command::IndirectProbesMissed { peer_id: 1 },
        ],
        1,
    );

    // Act
    let kept = liveness.of_members(&Members::new(vec![0, 2]));

    // Assert
    assert!(kept.suspected().is_empty());
    assert_eq!(kept.last_probed(), None);
}

// A member the change keeps keeps its liveness: still suspected, still the
// member probed last.
#[test]
fn of_members_keeps_whom_the_node_still_counts() {
    // Arrange
    let (_, liveness) = stepped(
        &[
            Command::ProbeDue,
            Command::IndirectProbesMissed { peer_id: 1 },
        ],
        1,
    );

    // Act
    let kept = liveness.of_members(&Members::new(vec![0, 1, 2, 3]));

    // Assert
    assert_eq!(kept.suspected(), &[1]);
    assert_eq!(kept.last_probed(), Some(1));
}

#[test]
fn step_of_a_command_not_of_liveness_is_none() {
    // Arrange
    let liveness = Liveness::default();

    // Act
    let stepped = liveness.step(Command::Probe, &members(), QuorumPolicy::new(2), &config(1));

    // Assert
    assert!(stepped.is_none());
}

#[test]
fn step_of_a_missed_probe_lists_the_others_to_ask() {
    // Arrange & Act
    let (outcomes, _) = stepped(&[Command::ProbeMissed { peer_id: 1 }], 1);

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::SendIndirectProbes {
            peer_id: 1,
            via: vec![2]
        }]
    );
}

#[test]
fn step_of_a_non_member_is_ignored() {
    // Arrange & Act
    let (outcomes, _) = stepped(&[Command::ProbeMissed { peer_id: 9 }], 1);

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 9 }]);
}

#[test]
fn step_of_a_period_missed_at_the_threshold_suspects_the_member() {
    // Arrange & Act
    let (outcomes, liveness) = stepped(
        &[
            Command::IndirectProbesMissed { peer_id: 1 },
            Command::IndirectProbesMissed { peer_id: 1 },
        ],
        2,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::PeerSuspected { peer_id: 1 }]);
    assert_eq!(liveness.suspected(), &[1]);
    assert!(liveness.misses().is_empty());
}

#[test]
fn step_of_a_period_missed_below_the_threshold_counts_it() {
    // Arrange & Act
    let (outcomes, liveness) = stepped(&[Command::IndirectProbesMissed { peer_id: 1 }], 2);

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::MissCounted {
            peer_id: 1,
            misses: 1
        }]
    );
    assert!(liveness.suspected().is_empty());
}

#[test]
fn step_of_a_probe_period_alone_has_no_one_to_probe() {
    // Arrange
    let liveness = Liveness::default();

    // Act
    let (outcomes, _) = liveness
        .step(
            Command::ProbeDue,
            &Members::new(vec![0]),
            QuorumPolicy::new(1),
            &config(1),
        )
        .expect("a command of liveness");

    // Assert
    assert_eq!(outcomes, vec![Outcome::NothingToProbe]);
}

#[test]
fn step_of_a_second_suspicion_loses_the_quorum() {
    // Arrange & Act
    let (outcomes, _) = stepped(
        &[
            Command::IndirectProbesMissed { peer_id: 1 },
            Command::IndirectProbesMissed { peer_id: 2 },
        ],
        1,
    );

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::PeerSuspected { peer_id: 2 }, Outcome::QuorumLost]
    );
}

#[test]
fn step_of_a_suspect_heard_from_revives_it() {
    // Arrange & Act
    let (outcomes, liveness) = stepped(
        &[
            Command::IndirectProbesMissed { peer_id: 1 },
            Command::PeerHeard { peer_id: 1 },
        ],
        1,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::PeerRevived { peer_id: 1 }]);
    assert!(liveness.suspected().is_empty());
}

#[test]
fn step_of_a_suspect_missed_again_changes_nothing() {
    // Arrange & Act
    let (outcomes, liveness) = stepped(
        &[
            Command::IndirectProbesMissed { peer_id: 1 },
            Command::ProbeMissed { peer_id: 1 },
        ],
        1,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::AlreadySuspected { peer_id: 1 }]);
    assert_eq!(liveness.suspected(), &[1]);
}

#[test]
fn step_of_an_alive_member_heard_from_forgets_its_misses() {
    // Arrange & Act
    let (outcomes, liveness) = stepped(
        &[
            Command::IndirectProbesMissed { peer_id: 1 },
            Command::PeerHeard { peer_id: 1 },
        ],
        3,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::AliveNoted { peer_id: 1 }]);
    assert!(liveness.misses().is_empty());
}

#[test]
fn step_of_the_node_itself_is_ignored_as_its_own() {
    // Arrange & Act
    let (outcomes, _) = stepped(&[Command::PeerHeard { peer_id: 0 }], 1);

    // Assert
    assert_eq!(outcomes, vec![Outcome::SelfIgnored { peer_id: 0 }]);
}

#[test]
fn suspected_lists_the_suspects_in_the_order_suspected() {
    // Arrange
    let (_, liveness) = stepped(
        &[
            Command::IndirectProbesMissed { peer_id: 2 },
            Command::IndirectProbesMissed { peer_id: 1 },
        ],
        1,
    );

    // Act
    let suspected = liveness.suspected();

    // Assert
    assert_eq!(suspected, &[2, 1]);
}
