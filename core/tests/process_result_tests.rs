// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::cluster_view::ClusterView;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;

extern crate alloc;

fn view() -> ClusterView {
    ClusterViewBuilder::new()
        .with_peer_state(PeerState::Collecting)
        .with_collecting_peers(vec![0, 2])
        .with_required_count(3)
        .build()
}

#[test]
fn into_cluster_view_of_accepted_returns_its_view() {
    // Arrange
    let result = ProcessResult::Accepted {
        cluster_view: view(),
        admissible: vec![Command::Probe],
        outcomes: vec![Outcome::ReadyAccepted { peer_id: 2 }],
    };

    // Act
    let cluster_view = result.into_cluster_view();

    // Assert
    assert_eq!(cluster_view, view());
}

#[test]
fn into_cluster_view_of_probed_returns_its_view() {
    // Arrange
    let result = ProcessResult::Probed {
        cluster_view: view(),
        admissible: vec![Command::Probe],
    };

    // Act
    let cluster_view = result.into_cluster_view();

    // Assert
    assert_eq!(cluster_view, view());
}

#[test]
fn into_cluster_view_of_rejected_returns_its_view() {
    // Arrange
    let result = ProcessResult::Rejected {
        cluster_view: view(),
        admissible: vec![Command::Probe],
    };

    // Act
    let cluster_view = result.into_cluster_view();

    // Assert
    assert_eq!(cluster_view, view());
}
