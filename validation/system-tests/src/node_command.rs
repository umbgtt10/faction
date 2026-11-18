// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::mpsc::Sender;

use faction::types::Epoch;

use crate::infra::fault_settings::FaultSettings;
use crate::membership::Membership;

pub enum NodeCommand {
    RequestJoin(u64, Sender<()>),
    Admit(u64, Sender<()>),
    Deny(u64, Sender<()>),
    ExpireDeadline(Sender<()>),
    RequestAddition(u64, Sender<()>),
    CommitAddition(u64, Epoch, Sender<()>),
    AbortAddition(u64, Sender<()>),
    RequestRemoval(u64, Sender<()>),
    CommitRemoval(u64, Epoch, Sender<()>),
    AbortRemoval(u64, Sender<()>),
    Decline(u64, Epoch, Sender<()>),
    ChangeRequiredCount(usize, Epoch, Sender<()>),
    Cut(u64, Sender<()>),
    Mend(u64, Sender<()>),
    Restart(Membership, Sender<()>),
    SetFaults(Box<FaultSettings>, Sender<()>),
    Shutdown,
}
