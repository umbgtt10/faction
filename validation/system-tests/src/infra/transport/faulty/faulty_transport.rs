// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::mem::take;

use faction::types::PeerId;
use faction_protocol::transport::Transport;
use faction_protocol::transport_message::TransportMessage;

use crate::infra::dice::Dice;
use crate::infra::live_policy::LivePolicy;
use crate::infra::logging::shared_writer::SharedWriter;
use crate::infra::transport::faulty::fault_policy::FaultPolicy;
use crate::infra::transport::faulty::link_cuts::LinkCuts;
use crate::infra::transport::faulty::message_kind::MessageKind;

const PARTITION_SALT: u64 = 0x5041_5254_494F_4E00;
const ASYMMETRIC_SALT: u64 = 0x4153_594D_4D00_0000;
const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;
const DELAY_STEPS: u64 = 3;
const REORDER_MAX_STEPS: u64 = 4;
const SEED_MIX: u64 = 0x9E37_79B9_7F4A_7C15;

/// A [`Transport`] decorator that injects transport faults per a seeded [`FaultPolicy`],
/// read live, so a test switches a fault on and heals it while a cluster runs, and
/// drops what crosses a link a test has cut.
///
/// Per-message faults (loss, duplication, delay, reorder, selective) are rolled per
/// `send` against a seeded RNG. The structural faults (partition, asymmetric) are
/// seeded *persistent* per-link cuts, so their percentage is the fraction of links cut
/// for the whole run. The [`LinkCuts`] are cut and mended while it runs: nothing is
/// sent to a peer cut off, and nothing it sends is received. Every fault is one line
/// of the node's log when it is given one, and a loud banner on stderr otherwise.
pub struct FaultyTransport {
    inner: Box<dyn Transport>,
    peer_id: PeerId,
    policy: LivePolicy<FaultPolicy>,
    cuts: LinkCuts,
    log: Option<SharedWriter>,
    dice: Dice,
    held: Vec<(u64, PeerId, TransportMessage)>,
}

impl FaultyTransport {
    #[must_use]
    pub fn new(
        peer_id: PeerId,
        policy: LivePolicy<FaultPolicy>,
        cuts: LinkCuts,
        log: Option<SharedWriter>,
        inner: Box<dyn Transport>,
    ) -> FaultyTransport {
        let dice = Dice::new(policy.current().seed ^ peer_id.wrapping_mul(SEED_MIX));
        FaultyTransport {
            inner,
            peer_id,
            policy,
            cuts,
            log,
            dice,
            held: Vec::new(),
        }
    }

    fn announce(&self, kind: &str, to: PeerId, message: &TransportMessage) {
        if let Some(log) = &self.log {
            log.write_line(&format!(
                "{{\"event\":\"fault\",\"fault\":\"{kind}\",\"node\":{},\"to\":{to},\"message\":\"{message:?}\"}}",
                self.peer_id
            ));
            return;
        }
        eprintln!();
        eprintln!("###############################################################");
        eprintln!("###  FAULT INJECTED >>> {kind}");
        eprintln!("###  peer {} -> peer {}   {:?}", self.peer_id, to, message);
        eprintln!("###############################################################");
        eprintln!();
    }

    fn is_asymmetric_cut(&self, to: PeerId, policy: &FaultPolicy) -> bool {
        Self::link_is_cut(policy, ASYMMETRIC_SALT, self.peer_id, to, policy.asymmetric)
    }

    fn is_partitioned(&self, to: PeerId, policy: &FaultPolicy) -> bool {
        let low = self.peer_id.min(to);
        let high = self.peer_id.max(to);
        Self::link_is_cut(policy, PARTITION_SALT, low, high, policy.partition)
    }

    fn link_is_cut(
        policy: &FaultPolicy,
        salt: u64,
        first: PeerId,
        second: PeerId,
        percent: u8,
    ) -> bool {
        if percent == 0 {
            return false;
        }
        if percent >= 100 {
            return true;
        }
        let mut hash = policy.seed ^ salt;
        hash = (hash ^ first).wrapping_mul(FNV_PRIME);
        hash = (hash ^ second).wrapping_mul(FNV_PRIME);
        hash ^= hash >> 29;
        (hash % 100) < u64::from(percent)
    }

    fn release_due(&mut self) {
        if self.held.is_empty() {
            return;
        }
        for (countdown, to, message) in take(&mut self.held) {
            if countdown <= 1 {
                self.inner.send(to, message);
            } else {
                self.held.push((countdown - 1, to, message));
            }
        }
    }

    fn sender(message: &TransportMessage) -> PeerId {
        match message {
            TransportMessage::Ping { from, .. }
            | TransportMessage::Ready { from, .. }
            | TransportMessage::Bootstrapped { from }
            | TransportMessage::Membership { from, .. }
            | TransportMessage::MembershipAcknowledged { from, .. }
            | TransportMessage::Probe { from, .. }
            | TransportMessage::ProbeAnswer { from, .. }
            | TransportMessage::IndirectProbe { from, .. }
            | TransportMessage::IndirectProbeAnswer { from, .. } => *from,
        }
    }
}

impl Transport for FaultyTransport {
    fn send(&mut self, to: PeerId, message: TransportMessage) {
        let policy = self.policy.current();
        self.release_due();

        if self.cuts.is_cut(to) {
            self.announce("CUT (link cut by the test)", to, &message);
            return;
        }
        if self.is_partitioned(to, &policy) {
            self.announce("PARTITION (link cut)", to, &message);
            return;
        }
        if self.is_asymmetric_cut(to, &policy) {
            self.announce("ASYMMETRIC (one-way link cut)", to, &message);
            return;
        }
        if MessageKind::of(&message) == policy.selective_target && self.dice.rolls(policy.selective)
        {
            self.announce("SELECTIVE DROP", to, &message);
            return;
        }
        if self.dice.rolls(policy.loss) {
            self.announce("LOSS (dropped)", to, &message);
            return;
        }
        if self.dice.rolls(policy.delay) {
            self.announce("DELAY (held)", to, &message);
            self.held.push((DELAY_STEPS, to, message));
            return;
        }
        if self.dice.rolls(policy.reorder) {
            let lag = 1 + self.dice.below(REORDER_MAX_STEPS);
            self.announce("REORDER (held out of order)", to, &message);
            self.held.push((lag, to, message));
            return;
        }
        if self.dice.rolls(policy.duplication) {
            self.announce("DUPLICATION (sent twice)", to, &message);
            self.inner.send(to, message.clone());
            self.inner.send(to, message);
            return;
        }

        self.inner.send(to, message);
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        self.release_due();
        self.inner
            .recv()
            .filter(|message| !self.cuts.is_cut(Self::sender(message)))
    }
}
