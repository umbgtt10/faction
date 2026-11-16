// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use serde::Serialize;

use faction_system_tests::infra::timer::faulty::clock_fault::ClockFault;
use faction_system_tests::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;

// A fault of one of a node's layers: its transport's, its timer's or its
// clock's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fault {
    Transport(FaultPolicy),
    Timer(TimerFaultPolicy),
    Clock(ClockFault),
}
