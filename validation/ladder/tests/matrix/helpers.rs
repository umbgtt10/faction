// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::matrix_table::MatrixTable;
use crate::support::spec_files::SpecFiles;
use alloc::vec;
use alloc::vec::Vec;
use faction::command::Command;
use faction::types::PeerId;

pub fn all_admissible() -> Vec<Command> {
    MatrixTable::new(SpecFiles::matrix()).admits("Pinging")
}

pub fn bootstrapped_admissible() -> Vec<Command> {
    MatrixTable::new(SpecFiles::matrix()).admits("Bootstrapped")
}

pub fn collecting_admissible() -> Vec<Command> {
    MatrixTable::new(SpecFiles::matrix()).admits("Collecting")
}

extern crate alloc;

pub fn initial_admissible() -> Vec<Command> {
    MatrixTable::new(SpecFiles::matrix()).admits("Initial")
}

pub fn join_approved(peer_id: PeerId) -> Command {
    Command::JoinApproved { peer_id }
}

pub fn join_rejected(peer_id: PeerId) -> Command {
    Command::JoinRejected { peer_id }
}

pub fn join_requested(peer_id: PeerId) -> Command {
    Command::JoinRequested { peer_id }
}

pub fn participation(peer_id: PeerId) -> Command {
    Command::ParticipationObserved { peer_id }
}

pub fn probe_only() -> Vec<Command> {
    vec![Command::Probe]
}

pub fn ready(peer_id: PeerId) -> Command {
    Command::ReadyObserved { peer_id }
}
