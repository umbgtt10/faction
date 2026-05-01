// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportMessage {
    Ping { from: PeerId },
    Ready { from: PeerId },
    Bootstrapped { from: PeerId },
}
