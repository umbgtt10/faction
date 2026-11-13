// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::members::Members;
use faction::outcome::Outcome;
use faction::states::probing::Probing;

extern crate alloc;

fn members() -> Members {
    Members::new(vec![0, 1, 2, 3, 4])
}

#[test]
fn helpers_leave_out_the_node_the_missed_member_and_the_suspects() {
    // Arrange & Act
    let helpers = Probing::helpers(&members(), 2, 0, &[3]);

    // Assert
    assert_eq!(helpers, vec![4, 1]);
}

#[test]
fn helpers_start_after_the_missed_member_and_wrap_around() {
    // Arrange & Act
    let helpers = Probing::helpers(&members(), 2, 0, &[]);

    // Assert
    assert_eq!(helpers, vec![3, 4, 1]);
}

#[test]
fn last_probed_of_a_node_that_never_probed_is_no_one() {
    // Arrange
    let probing = Probing::default();

    // Act
    let last_probed = probing.last_probed();

    // Assert
    assert_eq!(last_probed, None);
}

#[test]
fn probe_after_the_last_member_wraps_past_the_node_to_the_first_other() {
    // Arrange
    let (_, probing) = [0; 4]
        .iter()
        .fold((vec![], Probing::default()), |(_, probing), _| {
            probing.probe(&members(), 0)
        });

    // Act
    let (outcomes, probed) = probing.probe(&members(), 0);

    // Assert
    assert_eq!(outcomes, vec![Outcome::SendProbe { peer_id: 1 }]);
    assert_eq!(probed.last_probed(), Some(1));
}

#[test]
fn probe_of_a_node_alone_has_no_one_to_probe() {
    // Arrange
    let probing = Probing::default();

    // Act
    let (outcomes, probed) = probing.probe(&Members::new(vec![0]), 0);

    // Assert
    assert_eq!(outcomes, vec![Outcome::NothingToProbe]);
    assert_eq!(probed.last_probed(), None);
}

#[test]
fn probe_with_no_one_probed_names_the_first_member_but_the_node() {
    // Arrange
    let probing = Probing::default();

    // Act
    let (outcomes, probed) = probing.probe(&Members::new(vec![3, 1, 2]), 3);

    // Assert
    assert_eq!(outcomes, vec![Outcome::SendProbe { peer_id: 1 }]);
    assert_eq!(probed.last_probed(), Some(1));
}
