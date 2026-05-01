// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_system_tests::cluster::Cluster;

#[test]
fn five_nodes_converge_to_bootstrapped() {
    // Arrange & Act
    let mut cluster = Cluster::new(5, 4);
    cluster.converge();

    // Assert
    assert!(cluster.is_bootstrapped());
}
