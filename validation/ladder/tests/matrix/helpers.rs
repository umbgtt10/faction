// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::matrix_table::MatrixTable;
use crate::support::spec_files::SpecFiles;
use alloc::vec;
use alloc::vec::Vec;
use faction::command::Command;
use faction::command_kind::CommandKind;
use faction::types::Epoch;

pub fn addition_aborted(peer_id: u64) -> Command {
    Command::AdditionAborted { peer_id }
}

pub fn addition_committed(peer_id: u64) -> Command {
    Command::AdditionCommitted { peer_id, epoch: 0 }
}

pub fn addition_requested(peer_id: u64) -> Command {
    Command::AdditionRequested { peer_id }
}

pub fn bootstrapped_admissible() -> Vec<CommandKind> {
    MatrixTable::new(SpecFiles::matrix()).admits("Bootstrapped")
}

pub fn collecting_admissible() -> Vec<CommandKind> {
    MatrixTable::new(SpecFiles::matrix()).admits("Collecting")
}

pub fn coordinating_admissible() -> Vec<CommandKind> {
    MatrixTable::new(SpecFiles::matrix()).admits("Coordinating")
}

extern crate alloc;

pub fn indirect_probes_missed(peer_id: u64) -> Command {
    Command::IndirectProbesMissed { peer_id }
}

pub fn initial_admissible() -> Vec<CommandKind> {
    MatrixTable::new(SpecFiles::matrix()).admits("Initial")
}

pub fn join_approved(peer_id: u64) -> Command {
    Command::JoinApproved { peer_id }
}

pub fn join_rejected(peer_id: u64) -> Command {
    Command::JoinRejected { peer_id }
}

pub fn join_requested(peer_id: u64) -> Command {
    Command::JoinRequested { peer_id }
}

// A membership of `members`, requiring `required_count`, at `epoch`.
pub fn membership(members: &[u64], required_count: usize, epoch: Epoch) -> Command {
    Command::MembershipReceived {
        members: members.to_vec(),
        required_count,
        epoch,
    }
}

pub fn membership_acknowledged(peer_id: u64) -> Command {
    Command::MembershipAcknowledged { peer_id, epoch: 0 }
}

pub fn participation(peer_id: u64) -> Command {
    Command::ParticipationObserved { peer_id, epoch: 0 }
}

pub fn peer_heard(peer_id: u64) -> Command {
    Command::PeerHeard { peer_id }
}

pub fn pinging_admissible() -> Vec<CommandKind> {
    MatrixTable::new(SpecFiles::matrix()).admits("Pinging")
}

pub fn probe_missed(peer_id: u64) -> Command {
    Command::ProbeMissed { peer_id }
}

pub fn probe_only() -> Vec<CommandKind> {
    vec![CommandKind::Probe]
}

pub fn proposal_declined(peer_id: u64) -> Command {
    Command::ProposalDeclined { peer_id, epoch: 0 }
}

pub fn ready(peer_id: u64) -> Command {
    Command::ReadyObserved { peer_id, epoch: 0 }
}

pub fn reconfiguring_admissible() -> Vec<CommandKind> {
    MatrixTable::new(SpecFiles::matrix()).admits("Reconfiguring")
}

pub fn removal_aborted(peer_id: u64) -> Command {
    Command::RemovalAborted { peer_id }
}

pub fn removal_committed(peer_id: u64) -> Command {
    Command::RemovalCommitted { peer_id, epoch: 0 }
}

pub fn removal_requested(peer_id: u64) -> Command {
    Command::RemovalRequested {
        peer_id,
        leaves: None,
    }
}

pub fn removed_admissible() -> Vec<CommandKind> {
    MatrixTable::new(SpecFiles::matrix()).admits("Removed")
}

pub fn required_count_changed(required_count: usize) -> Command {
    Command::RequiredCountChanged {
        required_count,
        epoch: 0,
    }
}
