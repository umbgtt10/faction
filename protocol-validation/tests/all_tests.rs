// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol_validation::protocol_harness::ProtocolHarness;

#[test]
fn harness_creates_correct_number_of_protocols() {
    let harness = ProtocolHarness::new(5, 4);

    assert_eq!(harness.peer_ids().len(), 5);
}
