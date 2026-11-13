// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The property tier, liveness: a node concluded with three of five required,
// given commands of liveness, membership and count at random, at a suspicion
// threshold drawn too. Between two changes of its members it probes every
// other member within one round; it suspects a member only once it has missed
// as many periods running as the threshold, no sign of life among them; and
// its quorum is held exactly while its members not suspected reach the count,
// every crossing said once; and, asked for removals too, it proposes a removal
// exactly when the quorum can spare the member -- a suspect, or one without
// whom the members not suspected still reach the count -- and refuses it
// otherwise.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;
use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;
use faction::quorum::Quorum;
use faction::quorum_policy::QuorumPolicy;
use faction::types::PeerId;
use proptest::prelude::*;

// The outcomes and the view a command is answered with.
fn answered(faction: &mut Faction, command: Command) -> (Vec<Outcome>, ClusterView) {
    match faction.process(command) {
        ProcessResult::Accepted {
            outcomes,
            cluster_view,
            ..
        } => (outcomes, cluster_view),
        ProcessResult::Rejected { cluster_view, .. }
        | ProcessResult::Probed { cluster_view, .. } => (Vec::new(), cluster_view),
    }
}

// Concluded: its own completion and two peers' readiness reach three.
fn concluded(threshold: usize) -> Faction {
    let mut faction = Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(3), threshold),
        Box::new(NoOpObserver),
    );
    for command in [
        Command::LocalParticipationCompleted,
        Command::ReadyObserved { peer_id: 1 },
        Command::ReadyObserved { peer_id: 2 },
    ] {
        let _ = faction.process(command);
    }
    faction
}

extern crate alloc;

fn liveness_command() -> impl Strategy<Value = Command> {
    let peer = 0u64..=6;
    prop_oneof![
        3 => Just(Command::ProbeDue),
        2 => peer.clone().prop_map(|peer_id| Command::PeerHeard { peer_id }),
        2 => peer.clone().prop_map(|peer_id| Command::ProbeMissed { peer_id }),
        3 => peer.clone().prop_map(|peer_id| Command::IndirectProbesMissed { peer_id }),
        1 => peer.prop_map(|peer_id| Command::JoinApproved { peer_id }),
        1 => (0usize..=6).prop_map(|required_count| Command::RequiredCountChanged { required_count }),
    ]
}

proptest! {
    #[test]
    fn every_other_member_is_probed_within_one_round_between_two_changes_of_the_members(
        threshold in 1usize..=3,
        commands in prop::collection::vec(liveness_command(), 0..160),
    ) {
        // Arrange
        let mut faction = concluded(threshold);
        let mut rounds: Vec<(Vec<PeerId>, Vec<PeerId>)> = vec![(vec![0, 1, 2, 3, 4], Vec::new())];

        // Act
        for command in commands {
            let (outcomes, cluster_view) = answered(&mut faction, command);
            let members = cluster_view.members().as_slice().to_vec();
            if rounds.last().is_some_and(|(round_members, _)| *round_members != members) {
                rounds.push((members, Vec::new()));
            }
            let probed = outcomes.iter().filter_map(|outcome| match outcome {
                Outcome::SendProbe { peer_id } => Some(*peer_id),
                _ => None,
            });
            if let Some((_, probes)) = rounds.last_mut() {
                probes.extend(probed);
            }
        }

        // Assert
        for (members, probes) in rounds {
            let others: BTreeSet<PeerId> = members.iter().copied().filter(|member| *member != 0).collect();
            for window in probes.windows(others.len()) {
                let probed: BTreeSet<PeerId> = window.iter().copied().collect();
                prop_assert_eq!(&probed, &others);
            }
        }
    }

    #[test]
    fn a_member_is_suspected_only_once_it_has_missed_as_many_periods_running_as_the_threshold(
        threshold in 1usize..=3,
        commands in prop::collection::vec(liveness_command(), 0..160),
    ) {
        // Arrange
        let mut faction = concluded(threshold);
        let mut missed: BTreeMap<PeerId, usize> = BTreeMap::new();

        // Act
        for command in commands {
            let (outcomes, _) = answered(&mut faction, command);

            // Assert
            for outcome in outcomes {
                match outcome {
                    Outcome::MissCounted { peer_id, misses } => {
                        let counted = missed.entry(peer_id).or_insert(0);
                        *counted += 1;
                        prop_assert_eq!(misses, *counted);
                        prop_assert!(misses < threshold);
                    }
                    Outcome::PeerSuspected { peer_id } => {
                        prop_assert_eq!(missed.remove(&peer_id).unwrap_or(0) + 1, threshold);
                    }
                    Outcome::AliveNoted { peer_id } | Outcome::PeerRevived { peer_id } => {
                        missed.remove(&peer_id);
                    }
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn quorum_is_held_exactly_while_the_members_not_suspected_reach_the_count(
        threshold in 1usize..=3,
        commands in prop::collection::vec(liveness_command(), 0..160),
    ) {
        // Arrange
        let mut faction = concluded(threshold);
        let mut held = true;

        // Act
        for command in commands {
            let (outcomes, cluster_view) = answered(&mut faction, command);

            // Assert
            let alive = cluster_view
                .members()
                .as_slice()
                .iter()
                .filter(|member| !cluster_view.suspected().contains(member))
                .count();
            let holds = alive >= cluster_view.required_count();
            prop_assert_eq!(cluster_view.quorum(), if holds { Quorum::Held } else { Quorum::Lost });
            prop_assert_eq!(outcomes.contains(&Outcome::QuorumLost), held && !holds);
            prop_assert_eq!(outcomes.contains(&Outcome::QuorumRestored), !held && holds);
            held = holds;
        }
    }

    #[test]
    fn a_removal_is_proposed_exactly_when_the_quorum_can_spare_the_member(
        threshold in 1usize..=3,
        commands in prop::collection::vec(removal_command(), 0..160),
    ) {
        // Arrange
        let mut faction = concluded(threshold);
        let (_, mut previous) = answered(&mut faction, Command::Probe);

        // Act
        for command in commands {
            let (outcomes, cluster_view) = answered(&mut faction, command);

            // Assert
            if let Command::RemovalRequested { peer_id } = command {
                if outcomes.contains(&Outcome::ProposeRemoval { peer_id }) {
                    prop_assert!(spares(&previous, peer_id));
                }
                if outcomes.contains(&Outcome::RemovalRefused { peer_id }) {
                    prop_assert!(!spares(&previous, peer_id));
                }
            }
            previous = cluster_view;
        }
    }
}

// The liveness commands, and removals requested, aborted and committed among
// them, so that a node asked again has returned to `Bootstrapped`.
fn removal_command() -> impl Strategy<Value = Command> {
    let peer = 0u64..=6;
    prop_oneof![
        8 => liveness_command(),
        3 => peer.clone().prop_map(|peer_id| Command::RemovalRequested { peer_id }),
        2 => peer.clone().prop_map(|peer_id| Command::RemovalAborted { peer_id }),
        1 => peer.prop_map(|peer_id| Command::RemovalCommitted { peer_id }),
    ]
}

// Whether the quorum a view shows can spare the member: it is suspected, or
// the members not suspected without it still reach the count.
fn spares(cluster_view: &ClusterView, peer_id: PeerId) -> bool {
    let suspected = cluster_view.suspected();
    suspected.contains(&peer_id)
        || cluster_view.members().without(peer_id).is_some_and(|rest| {
            rest.as_slice()
                .iter()
                .filter(|member| !suspected.contains(member))
                .count()
                >= cluster_view.required_count()
        })
}
