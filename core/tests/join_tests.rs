// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;

fn fresh() -> Faction {
    Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4)),
        Box::new(NoOpObserver),
    )
}

fn process(faction: &mut Faction, command: Command) -> Vec<Outcome> {
    match faction.process(command) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Rejected { .. } | ProcessResult::Probed { .. } => Vec::new(),
    }
}

fn probe(faction: &mut Faction) -> ClusterView {
    match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    }
}

#[test]
fn participation_from_an_admitted_peer_is_accepted() {
    // Arrange
    let mut faction = fresh();
    let admitted = process(&mut faction, Command::JoinApproved { peer_id: 99 });

    // Act
    let accepted = process(&mut faction, Command::ParticipationObserved { peer_id: 99 });

    // Assert
    assert_eq!(admitted, vec![Outcome::MemberAdmitted { peer_id: 99 }]);
    assert_eq!(
        accepted,
        vec![Outcome::ParticipationAccepted { peer_id: 99 }]
    );
}

#[test]
fn admission_does_not_retroactively_count_earlier_participation() {
    // Arrange
    let mut faction = fresh();
    let ignored = process(&mut faction, Command::ParticipationObserved { peer_id: 99 });
    let admitted = process(&mut faction, Command::JoinApproved { peer_id: 99 });

    // Act
    let after_admit = probe(&mut faction);
    let accepted = process(&mut faction, Command::ParticipationObserved { peer_id: 99 });

    // Assert
    assert_eq!(ignored, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(admitted, vec![Outcome::MemberAdmitted { peer_id: 99 }]);
    assert_eq!(after_admit.pinging_peers().len(), 0);
    assert_eq!(
        accepted,
        vec![Outcome::ParticipationAccepted { peer_id: 99 }]
    );
}

#[test]
fn join_rejected_does_not_remove_an_existing_member() {
    // Arrange
    let mut faction = fresh();
    let _ = process(&mut faction, Command::JoinApproved { peer_id: 99 });
    let _ = process(&mut faction, Command::ParticipationObserved { peer_id: 99 });

    // Act
    let denied = process(&mut faction, Command::JoinRejected { peer_id: 99 });
    let still_member = process(&mut faction, Command::ParticipationObserved { peer_id: 99 });

    // Assert
    assert_eq!(denied, vec![Outcome::JoinDenied { peer_id: 99 }]);
    assert_eq!(
        still_member,
        vec![Outcome::DuplicateParticipationIgnored { peer_id: 99 }]
    );
}

#[test]
fn approving_the_local_member_is_a_duplicate() {
    // Arrange & Act
    let mut faction = fresh();
    let outcomes = process(&mut faction, Command::JoinApproved { peer_id: 0 });

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateMemberIgnored { peer_id: 0 }]
    );
}

#[test]
fn admission_does_not_change_the_quorum_threshold() {
    // Arrange
    let mut faction = fresh();

    // Act
    let admitted = process(&mut faction, Command::JoinApproved { peer_id: 99 });
    let required = probe(&mut faction).required_count();

    // Assert
    assert_eq!(admitted, vec![Outcome::MemberAdmitted { peer_id: 99 }]);
    assert_eq!(required, 4);
}

#[test]
fn a_repeated_join_request_is_forwarded_each_time() {
    // Arrange
    let mut faction = fresh();

    // Act
    let first = process(&mut faction, Command::JoinRequested { peer_id: 99 });
    let second = process(&mut faction, Command::JoinRequested { peer_id: 99 });

    // Assert
    assert_eq!(first, vec![Outcome::EmitJoinRequest { peer_id: 99 }]);
    assert_eq!(second, vec![Outcome::EmitJoinRequest { peer_id: 99 }]);
}
