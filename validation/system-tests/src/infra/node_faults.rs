// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::infra::live_policy::LivePolicy;
use crate::infra::timer::faulty::clock_fault::ClockFault;
use crate::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;
use crate::infra::transport::faulty::fault_policy::FaultPolicy;
use crate::infra::transport::faulty::link_cuts::LinkCuts;

// The faults of one node, each a live policy its fault layer reads -- its
// transport's, its timer's and its clock's -- and the links a test has cut.
// The default is a faithful node.
#[derive(Debug, Clone, Default)]
pub struct NodeFaults {
    pub transport: LivePolicy<FaultPolicy>,
    pub timer: LivePolicy<TimerFaultPolicy>,
    pub clock: LivePolicy<ClockFault>,
    pub cuts: LinkCuts,
}
