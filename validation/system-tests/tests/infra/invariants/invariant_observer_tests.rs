// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::cluster_view::ClusterView;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::config::Config;
use faction::faction::Faction;
use faction::members::Members;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::types::Epoch;
use faction_system_tests::infra::invariants::breach::Breach;
use faction_system_tests::infra::invariants::breaches::Breaches;
use faction_system_tests::infra::invariants::invariant_observer::InvariantObserver;
use std::sync::Arc;
use std::sync::Mutex;

fn accepted(cluster_view: ClusterView, outcomes: Vec<Outcome>) -> ProcessResult {
    ProcessResult::Accepted {
        cluster_view,
        admissible: Vec::new(),
        outcomes,
    }
}

struct CountingObserver {
    calls: Arc<Mutex<usize>>,
}

impl Observer for CountingObserver {
    fn observe(&mut self, _: &Command, _: &ProcessResult) {
        *self.calls.lock().unwrap() += 1;
    }
}

fn observer(epoch: Epoch, breaches: &Breaches) -> InvariantObserver {
    InvariantObserver::new(
        0,
        Some(vec![0, 1, 2]),
        epoch,
        breaches.clone(),
        Box::new(NoOpObserver),
    )
}

fn probed(cluster_view: ClusterView) -> ProcessResult {
    ProcessResult::Probed {
        cluster_view,
        admissible: Vec::new(),
    }
}

fn rejected(cluster_view: ClusterView) -> ProcessResult {
    ProcessResult::Rejected {
        cluster_view,
        admissible: Vec::new(),
    }
}

fn removed() -> ClusterView {
    ClusterViewBuilder::from_view(&view(0))
        .with_peer_state(PeerState::Removed)
        .build()
}

// The view of node 0 of three, all required, at `epoch`.
fn view(epoch: Epoch) -> ClusterView {
    let config = Config::new(0, vec![0, 1, 2], QuorumPolicy::new(3), 1, epoch, false);
    let ProcessResult::Probed { cluster_view, .. } =
        Faction::new(config, Box::new(NoOpObserver)).process(Command::Probe)
    else {
        panic!("a fresh node answers a probe");
    };
    cluster_view
}

fn with_members(members: Vec<u64>) -> ClusterView {
    ClusterViewBuilder::from_view(&view(1))
        .with_members(Members::new(members))
        .build()
}

#[test]
fn observe_by_a_newcomer_learns_its_members_from_its_first_view() {
    // Arrange
    let breaches = Breaches::default();
    let mut observer = InvariantObserver::new(3, None, 0, breaches.clone(), Box::new(NoOpObserver));
    observer.observe(&Command::Probe, &probed(with_members(vec![0, 1, 3])));
    let learned = breaches.first();

    // Act
    observer.observe(&Command::Probe, &probed(with_members(vec![0, 1, 3, 7])));

    // Assert
    assert_eq!(learned, None);
    assert_eq!(
        breaches.first().map(|breach| breach.invariant),
        Some("a member it was never told of")
    );
}

#[test]
fn observe_by_a_removed_node_of_a_probe_reports_nothing() {
    // Arrange
    let breaches = Breaches::default();
    let mut observer = observer(0, &breaches);
    observer.observe(&Command::Probe, &probed(removed()));

    // Act
    observer.observe(&Command::Probe, &probed(removed()));

    // Assert
    assert_eq!(breaches.first(), None);
}

#[test]
fn observe_by_a_removed_node_of_a_refused_command_reports_nothing() {
    // Arrange
    let breaches = Breaches::default();
    let mut observer = observer(0, &breaches);
    observer.observe(&Command::Probe, &probed(removed()));
    let ping = Command::ParticipationObserved {
        peer_id: 1,
        epoch: 0,
    };

    // Act
    observer.observe(&ping, &rejected(removed()));

    // Assert
    assert_eq!(breaches.first(), None);
}

#[test]
fn observe_by_a_removed_node_of_an_answered_command_reports_it() {
    // Arrange
    let breaches = Breaches::default();
    let mut observer = observer(0, &breaches);
    observer.observe(&Command::Probe, &probed(removed()));
    let ping = Command::ParticipationObserved {
        peer_id: 1,
        epoch: 0,
    };

    // Act
    observer.observe(
        &ping,
        &accepted(
            removed(),
            vec![Outcome::ParticipationAccepted { peer_id: 1 }],
        ),
    );

    // Assert
    assert_eq!(
        breaches.first(),
        Some(Breach {
            peer_id: 0,
            invariant: "a removed node answered",
            command: ping,
        })
    );
}

#[test]
fn observe_of_a_claim_from_behind_counted_reports_it() {
    // Arrange
    let breaches = Breaches::default();
    let mut observer = observer(3, &breaches);
    let ready = Command::ReadyObserved {
        peer_id: 1,
        epoch: 2,
    };

    // Act
    observer.observe(
        &ready,
        &accepted(view(3), vec![Outcome::ReadyAccepted { peer_id: 1 }]),
    );

    // Assert
    assert_eq!(
        breaches.first().map(|breach| breach.invariant),
        Some("a claim from behind counted")
    );
}

#[test]
fn observe_of_a_claim_from_behind_refused_reports_nothing() {
    // Arrange
    let breaches = Breaches::default();
    let mut observer = observer(3, &breaches);
    let ping = Command::ParticipationObserved {
        peer_id: 1,
        epoch: 2,
    };

    // Act
    observer.observe(
        &ping,
        &accepted(
            view(3),
            vec![
                Outcome::StaleEpochIgnored {
                    peer_id: 1,
                    epoch: 2,
                },
                Outcome::SendMembership { peer_id: 1 },
            ],
        ),
    );

    // Assert
    assert_eq!(breaches.first(), None);
}

#[test]
fn observe_of_a_conclusion_short_of_the_required_count_reports_it() {
    // Arrange
    let breaches = Breaches::default();
    let mut observer = observer(0, &breaches);
    let concluded = ClusterViewBuilder::from_view(&view(0))
        .with_collecting_peers(vec![0, 1])
        .build();

    // Act
    observer.observe(
        &Command::ReadyObserved {
            peer_id: 1,
            epoch: 0,
        },
        &accepted(
            concluded,
            vec![
                Outcome::ReadyAccepted { peer_id: 1 },
                Outcome::Concluded {
                    mode: Conclusion::Bootstrapped,
                },
            ],
        ),
    );

    // Assert
    assert_eq!(
        breaches.first().map(|breach| breach.invariant),
        Some("concluded short of the required count")
    );
}

#[test]
fn observe_of_a_faithful_bootstrap_reports_nothing() {
    // Arrange
    let breaches = Breaches::default();
    let config = Config::new(0, vec![0, 1], QuorumPolicy::new(2), 1, 0, false);
    let mut faction = Faction::new(config, Box::new(observer(0, &breaches)));
    let commands = [
        Command::ParticipationObserved {
            peer_id: 1,
            epoch: 0,
        },
        Command::LocalParticipationCompleted,
        Command::ReadyObserved {
            peer_id: 1,
            epoch: 0,
        },
        Command::Probe,
    ];

    // Act
    let results: Vec<ProcessResult> = commands
        .into_iter()
        .map(|command| faction.process(command))
        .collect();

    // Assert
    assert!(matches!(
        results.last(),
        Some(ProcessResult::Probed { cluster_view, .. })
            if cluster_view.peer_state() == PeerState::Bootstrapped
    ));
    assert_eq!(breaches.first(), None);
}

#[test]
fn observe_of_a_fallen_epoch_reports_it() {
    // Arrange
    let breaches = Breaches::default();
    let mut observer = observer(3, &breaches);

    // Act
    observer.observe(&Command::Probe, &probed(view(2)));

    // Assert
    assert_eq!(
        breaches.first(),
        Some(Breach {
            peer_id: 0,
            invariant: "the epoch fell",
            command: Command::Probe,
        })
    );
}

#[test]
fn observe_of_a_member_named_by_its_commit_reports_nothing() {
    // Arrange
    let breaches = Breaches::default();
    let mut observer = observer(0, &breaches);

    // Act
    observer.observe(
        &Command::AdditionCommitted {
            peer_id: 9,
            epoch: 1,
        },
        &accepted(
            with_members(vec![0, 1, 2, 9]),
            vec![Outcome::MemberAdmitted { peer_id: 9 }],
        ),
    );

    // Assert
    assert_eq!(breaches.first(), None);
}

#[test]
fn observe_of_a_member_never_told_of_reports_it() {
    // Arrange
    let breaches = Breaches::default();
    let mut observer = observer(0, &breaches);

    // Act
    observer.observe(&Command::Probe, &probed(with_members(vec![0, 1, 2, 9])));

    // Assert
    assert_eq!(
        breaches.first().map(|breach| breach.invariant),
        Some("a member it was never told of")
    );
}

#[test]
fn observe_passes_every_call_on_to_the_inner_observer() {
    // Arrange
    let calls = Arc::new(Mutex::new(0));
    let mut observer = InvariantObserver::new(
        0,
        Some(vec![0, 1, 2]),
        3,
        Breaches::default(),
        Box::new(CountingObserver {
            calls: calls.clone(),
        }),
    );

    // Act
    observer.observe(&Command::Probe, &probed(view(3)));
    observer.observe(&Command::Probe, &probed(view(2)));

    // Assert
    assert_eq!(*calls.lock().unwrap(), 2);
}
