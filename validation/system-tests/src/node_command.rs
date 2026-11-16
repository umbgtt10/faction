// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::mpsc::Sender;

use faction::types::Epoch;
use faction::types::PeerId;

use crate::membership::Membership;

pub enum NodeCommand {
    RequestJoin(PeerId, Sender<()>),
    Admit(PeerId, Sender<()>),
    Deny(PeerId, Sender<()>),
    ExpireDeadline(Sender<()>),
    RequestAddition(PeerId, Sender<()>),
    CommitAddition(PeerId, Epoch, Sender<()>),
    AbortAddition(PeerId, Sender<()>),
    RequestRemoval(PeerId, Sender<()>),
    CommitRemoval(PeerId, Epoch, Sender<()>),
    AbortRemoval(PeerId, Sender<()>),
    ChangeRequiredCount(usize, Epoch, Sender<()>),
    Cut(PeerId, Sender<()>),
    Mend(PeerId, Sender<()>),
    Restart(Membership, Sender<()>),
    Shutdown,
}
