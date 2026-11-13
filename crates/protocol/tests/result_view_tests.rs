// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::cluster_view::ClusterView;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction_protocol::result_view::ResultView;

extern crate alloc;

fn view() -> ClusterView {
    ClusterViewBuilder::default()
        .with_peer_state(PeerState::Collecting)
        .with_collecting_peers(vec![0, 2])
        .with_required_count(3)
        .build()
}

#[test]
fn of_accepted_returns_its_view() {
    // Arrange
    let result = ProcessResult::Accepted {
        cluster_view: view(),
        admissible: vec![Command::Probe],
        outcomes: vec![Outcome::ReadyAccepted { peer_id: 2 }],
    };

    // Act
    let cluster_view = ResultView::of(result);

    // Assert
    assert_eq!(cluster_view, view());
}

#[test]
fn of_probed_returns_its_view() {
    // Arrange
    let result = ProcessResult::Probed {
        cluster_view: view(),
        admissible: vec![Command::Probe],
    };

    // Act
    let cluster_view = ResultView::of(result);

    // Assert
    assert_eq!(cluster_view, view());
}

#[test]
fn of_rejected_returns_its_view() {
    // Arrange
    let result = ProcessResult::Rejected {
        cluster_view: view(),
        admissible: vec![Command::Probe],
    };

    // Act
    let cluster_view = ResultView::of(result);

    // Assert
    assert_eq!(cluster_view, view());
}
