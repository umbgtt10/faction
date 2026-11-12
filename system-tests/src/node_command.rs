// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::mpsc::Sender;

use faction::types::PeerId;

pub enum NodeCommand {
    RequestJoin(PeerId, Sender<()>),
    Admit(PeerId, Sender<()>),
    Deny(PeerId, Sender<()>),
    ExpireDeadline(Sender<()>),
    Shutdown,
}
