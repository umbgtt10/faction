// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::transport::faulty::link_cuts::LinkCuts;

#[test]
fn cut_of_a_peer_is_cut_and_in_every_clone() {
    // Arrange
    let cuts = LinkCuts::default();
    let shared = cuts.clone();

    // Act
    cuts.cut(3);

    // Assert
    assert!(shared.is_cut(3));
}

#[test]
fn is_cut_of_a_peer_never_cut_is_false() {
    // Arrange
    let cuts = LinkCuts::default();
    cuts.cut(3);

    // Act
    let cut = cuts.is_cut(4);

    // Assert
    assert!(!cut);
}

#[test]
fn mend_of_a_cut_peer_is_no_longer_cut() {
    // Arrange
    let cuts = LinkCuts::default();
    cuts.cut(3);

    // Act
    cuts.mend(3);

    // Assert
    assert!(!cuts.is_cut(3));
}
