// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

use faction_system_tests::infra::dice::Dice;
use faction_system_tests::infra::timer::faulty::clock_fault::ClockFault;
use faction_system_tests::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;
use faction_system_tests::infra::transport::faulty::message_kind::MessageKind;

use crate::model::fault::Fault;
use crate::model::fault_window::FaultWindow;

const FAST_2X_PPM: i64 = 1_000_000;
const KINDS: [MessageKind; 9] = [
    MessageKind::Ping,
    MessageKind::Ready,
    MessageKind::Bootstrapped,
    MessageKind::Membership,
    MessageKind::MembershipAcknowledged,
    MessageKind::Probe,
    MessageKind::ProbeAnswer,
    MessageKind::IndirectProbe,
    MessageKind::IndirectProbeAnswer,
];
const MAX_DRIFT_PPM: u64 = 1_000;
const MAX_GRAIN_MS: u64 = 5;
const MAX_WINDOWS: u64 = 6;
const MIN_PERCENT: u64 = 10;
const MIN_SPAN: u64 = 10;
const PERCENT_SPAN: u64 = 50;
const SLOW_21X_PPM: i64 = -952_381;
const TIMER_FAULTS: u64 = 6;
const TRANSPORT_FAULTS: u64 = 16;

// A scenario's fault windows, each one fault of one node's transport, timer
// or clock, as Embassy's boards and QEMU runs can produce them. A timer side
// dies only on a node a restart may heal.
pub struct FaultDraw {
    dice: Dice,
}

impl FaultDraw {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            dice: Dice::new(seed),
        }
    }

    // Windows over `peers` opening within `chaos_rounds`.
    pub fn windows(
        &mut self,
        peers: &[u64],
        restartable: &[u64],
        chaos_rounds: u64,
    ) -> Vec<FaultWindow> {
        let count = self.dice.below(MAX_WINDOWS + 1);
        (0..count)
            .map(|_| {
                let peer_id = peers[self.dice.below(peers.len() as u64) as usize];
                let from = self.dice.below(chaos_rounds);
                let until = from + MIN_SPAN + self.dice.below(chaos_rounds);
                let fault = match self.dice.below(3) {
                    0 => Fault::Transport(self.transport()),
                    1 => Fault::Timer(self.timer(restartable.contains(&peer_id))),
                    _ => Fault::Clock(self.clock()),
                };
                FaultWindow {
                    peer_id,
                    from,
                    until,
                    fault,
                }
            })
            .collect()
    }

    // A clock 21 times slow, as QEMU's was, or twice fast; an oscillator's
    // drift; or a coarse tick.
    fn clock(&mut self) -> ClockFault {
        let drift = 1 + self.dice.below(MAX_DRIFT_PPM) as i64;
        match self.dice.below(4) {
            0 => ClockFault {
                drift_ppm: SLOW_21X_PPM,
                ..ClockFault::default()
            },
            1 => ClockFault {
                drift_ppm: FAST_2X_PPM,
                ..ClockFault::default()
            },
            2 => ClockFault {
                drift_ppm: if self.dice.rolls(50) { drift } else { -drift },
                ..ClockFault::default()
            },
            _ => ClockFault {
                grain: Duration::from_millis(1 + self.dice.below(MAX_GRAIN_MS)),
                ..ClockFault::default()
            },
        }
    }

    fn percent(&mut self) -> u8 {
        (MIN_PERCENT + self.dice.below(PERCENT_SPAN)) as u8
    }

    fn timer(&mut self, may_die: bool) -> TimerFaultPolicy {
        let base = TimerFaultPolicy {
            seed: self.dice.draw(),
            ..TimerFaultPolicy::default()
        };
        let percent = self.percent();
        let families = if may_die {
            TIMER_FAULTS + 1
        } else {
            TIMER_FAULTS
        };
        match self.dice.below(families) {
            0 => TimerFaultPolicy {
                late: percent,
                ..base
            },
            1 => TimerFaultPolicy {
                reorder: percent,
                ..base
            },
            2 => TimerFaultPolicy {
                duplicate: percent,
                ..base
            },
            3 => TimerFaultPolicy {
                lost_cancel: percent,
                ..base
            },
            4 => TimerFaultPolicy {
                replace: true,
                ..base
            },
            5 => TimerFaultPolicy {
                starved: true,
                ..base
            },
            _ => TimerFaultPolicy { dead: true, ..base },
        }
    }

    fn transport(&mut self) -> FaultPolicy {
        let base = FaultPolicy {
            seed: self.dice.draw(),
            ..FaultPolicy::default()
        };
        let percent = self.percent();
        let few = 1 + self.dice.below(3) as u8;
        let kind = KINDS[self.dice.below(KINDS.len() as u64) as usize];
        match self.dice.below(TRANSPORT_FAULTS) {
            0 => FaultPolicy {
                loss: percent,
                ..base
            },
            1 => FaultPolicy {
                duplication: percent,
                ..base
            },
            2 => FaultPolicy {
                delay: percent,
                ..base
            },
            3 => FaultPolicy {
                reorder: percent,
                ..base
            },
            4 => FaultPolicy {
                partition: percent,
                ..base
            },
            5 => FaultPolicy {
                asymmetric: percent,
                ..base
            },
            6 => FaultPolicy {
                selective: percent,
                selective_target: kind,
                ..base
            },
            7 => FaultPolicy {
                misdelivery: percent / 2,
                ..base
            },
            8 => FaultPolicy {
                first_contact_lost: few,
                ..base
            },
            9 => FaultPolicy {
                inbox: usize::from(few),
                ..base
            },
            10 => FaultPolicy {
                size_loss: percent,
                ..base
            },
            11 => FaultPolicy {
                burst_loss: percent / 2,
                ..base
            },
            12 => FaultPolicy { mute: true, ..base },
            13 => FaultPolicy { deaf: true, ..base },
            14 => FaultPolicy {
                starved: true,
                ..base
            },
            _ => FaultPolicy {
                head_of_line: true,
                ..base
            },
        }
    }
}
