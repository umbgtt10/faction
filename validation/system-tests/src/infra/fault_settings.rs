// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use serde::Serialize;

use crate::infra::timer::faulty::clock_fault::ClockFault;
use crate::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;
use crate::infra::transport::faulty::fault_policy::FaultPolicy;

// The faults one node's layers are set to at once -- its transport's, its
// timer's and its clock's -- as a value a node in another process can be
// sent. The default is a faithful node.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultSettings {
    pub transport: FaultPolicy,
    pub timer: TimerFaultPolicy,
    pub clock: ClockFault,
}
