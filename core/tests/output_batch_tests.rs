// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;
use faction::cluster_readiness_output::ClusterReadinessOutput;
use faction::output_batch::OutputBatch;
use faction::readiness_exit_mode::ReadinessExitMode;

#[test]
fn new_batch_is_empty() {
    // Arrange & Act
    let batch = OutputBatch::new();

    // Assert
    assert_eq!(batch.len(), 0);
    assert!(batch.is_empty());
    assert_eq!(batch.get(0), None);
}

#[test]
fn from_preserves_single_output() {
    // Act
    let batch = OutputBatch::from(vec![ClusterReadinessOutput::BroadcastLocalReady]);

    // Assert
    assert_eq!(batch.len(), 1);
    assert!(!batch.is_empty());
    assert_eq!(
        batch.get(0),
        Some(ClusterReadinessOutput::BroadcastLocalReady)
    );
    assert_eq!(batch.get(1), None);
}

#[test]
fn from_preserves_two_outputs_in_order() {
    // Act
    let batch = OutputBatch::from(vec![
        ClusterReadinessOutput::LocalParticipationCompleted,
        ClusterReadinessOutput::BroadcastLocalReady,
    ]);

    // Assert
    assert_eq!(batch.len(), 2);
    assert_eq!(
        batch.get(0),
        Some(ClusterReadinessOutput::LocalParticipationCompleted)
    );
    assert_eq!(
        batch.get(1),
        Some(ClusterReadinessOutput::BroadcastLocalReady)
    );
    assert_eq!(batch.get(2), None);
}

#[test]
fn from_preserves_three_outputs_in_order() {
    // Act
    let batch = OutputBatch::from(vec![
        ClusterReadinessOutput::ReadyAccepted { peer_id: 3 },
        ClusterReadinessOutput::ReadyQuorumReached,
        ClusterReadinessOutput::ReadinessExited {
            mode: ReadinessExitMode::Quorum,
        },
    ]);

    // Assert
    assert_eq!(batch.len(), 3);
    assert_eq!(
        batch.get(0),
        Some(ClusterReadinessOutput::ReadyAccepted { peer_id: 3 })
    );
    assert_eq!(
        batch.get(1),
        Some(ClusterReadinessOutput::ReadyQuorumReached)
    );
    assert_eq!(
        batch.get(2),
        Some(ClusterReadinessOutput::ReadinessExited {
            mode: ReadinessExitMode::Quorum,
        })
    );
    assert_eq!(batch.get(3), None);
}

#[test]
fn from_preserves_existing_output_order() {
    // Act
    let batch = OutputBatch::from(vec![
        ClusterReadinessOutput::LocalParticipationCompleted,
        ClusterReadinessOutput::BroadcastLocalReady,
        ClusterReadinessOutput::ReadyQuorumReached,
    ]);

    // Assert
    assert_eq!(batch.len(), 3);
    assert_eq!(
        batch.get(0),
        Some(ClusterReadinessOutput::LocalParticipationCompleted)
    );
    assert_eq!(
        batch.get(1),
        Some(ClusterReadinessOutput::BroadcastLocalReady)
    );
    assert_eq!(
        batch.get(2),
        Some(ClusterReadinessOutput::ReadyQuorumReached)
    );
    assert_eq!(batch.get(3), None);
}

#[test]
fn get_returns_none_for_out_of_bounds_index() {
    // Arrange & Act
    let batch = OutputBatch::from(vec![ClusterReadinessOutput::BroadcastLocalReady]);

    // Assert
    assert_eq!(batch.get(1), None);
    assert_eq!(batch.get(99), None);
}
