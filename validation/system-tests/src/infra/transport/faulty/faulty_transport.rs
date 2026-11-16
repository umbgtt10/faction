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
use crate::infra::transport::faulty::first_contact::FirstContact;
use crate::infra::transport::faulty::inbox::Inbox;
use crate::infra::transport::faulty::link_cuts::LinkCuts;
use crate::infra::transport::faulty::message_kind::MessageKind;
use crate::infra::transport::faulty::misdelivery::Misdelivery;
use crate::infra::transport::faulty::wedge::Wedge;

const PARTITION_SALT: u64 = 0x5041_5254_494F_4E00;
const ASYMMETRIC_SALT: u64 = 0x4153_594D_4D00_0000;
const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;
const INBOX_SALT: u64 = 0x494E_424F_5800_0000;
const MISDELIVERY_SALT: u64 = 0x4D49_5344_454C_0000;
const DELAY_STEPS: u64 = 3;
const REORDER_MAX_STEPS: u64 = 4;
const SEED_MIX: u64 = 0x9E37_79B9_7F4A_7C15;

/// A [`Transport`] decorator that injects transport faults per a seeded [`FaultPolicy`],
/// read live, so a test switches a fault on and heals it while a cluster runs, and
/// drops what crosses a link a test has cut.
///
/// Per-message faults (loss, duplication, delay, reorder, selective, misdelivery) are
/// rolled per `send` against a seeded RNG. The structural faults (partition,
/// asymmetric) are seeded *persistent* per-link cuts, so their percentage is the
/// fraction of links cut for the whole run. What a board can produce besides: a
/// node mute, deaf or starved, sends wedged behind one to an unreachable peer, the
/// first messages to each peer lost, and a receive ring of bounded capacity losing
/// messages when busy, by size and by backlog (see [`Inbox`]). The [`LinkCuts`] are
/// cut and mended while it runs: nothing is sent to a peer cut off, and nothing it
/// sends is received. Every fault is one line of the node's log when it is given
/// one, and a loud banner on stderr otherwise.
pub struct FaultyTransport {
    inner: Box<dyn Transport>,
    peer_id: PeerId,
    policy: LivePolicy<FaultPolicy>,
    cuts: LinkCuts,
    log: Option<SharedWriter>,
    dice: Dice,
    held: Vec<(u64, PeerId, TransportMessage)>,
    stalled: Vec<(PeerId, TransportMessage)>,
    wedge: Wedge,
    first_contact: FirstContact,
    misdelivery: Misdelivery,
    inbox: Inbox,
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
        let seed = policy.current().seed ^ peer_id.wrapping_mul(SEED_MIX);
        FaultyTransport {
            inner,
            peer_id,
            policy,
            cuts,
            log,
            dice: Dice::new(seed),
            held: Vec::new(),
            stalled: Vec::new(),
            wedge: Wedge::default(),
            first_contact: FirstContact::default(),
            misdelivery: Misdelivery::new(seed ^ MISDELIVERY_SALT),
            inbox: Inbox::new(seed ^ INBOX_SALT),
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

    // A message the inner transport received, refused by a deaf node, dropped
    // from a peer cut off, or put through the receive ring.
    fn arrive(&mut self, message: TransportMessage, policy: &FaultPolicy) {
        if policy.deaf {
            self.announce("DEAF (nothing received)", self.peer_id, &message);
            return;
        }
        if self.cuts.is_cut(Self::sender(&message)) {
            return;
        }
        if let Some((fault, lost)) = self.inbox.arrive(message, policy) {
            self.announce(fault, self.peer_id, &lost);
        }
    }

    // What a starved node held, sent in order now that it is fed.
    fn flush_stalled(&mut self) {
        for (to, message) in take(&mut self.stalled) {
            self.send(to, message);
        }
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
        if policy.starved {
            self.stalled.push((to, message));
            return;
        }
        self.flush_stalled();

        if policy.mute {
            self.announce("MUTE (nothing sent)", to, &message);
            return;
        }
        if self.wedge.blocks(to, &policy) {
            self.announce(
                "HEAD-OF-LINE (wedged behind an unreachable peer)",
                to,
                &message,
            );
            return;
        }
        if policy.unreachable.contains(&to) {
            self.announce("UNREACHABLE (no neighbour answers)", to, &message);
            return;
        }
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
        if self.first_contact.loses(to, policy.first_contact_lost) {
            self.announce("FIRST CONTACT (lost to resolution)", to, &message);
            return;
        }
        let addressed = to;
        let to = self.misdelivery.addressee(addressed, policy.misdelivery);
        if to != addressed {
            self.announce("MISDELIVERY (reached another peer)", to, &message);
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
        let policy = self.policy.current();
        self.release_due();
        if policy.starved {
            return None;
        }
        self.flush_stalled();
        while let Some(message) = self.inner.recv() {
            self.arrive(message, &policy);
        }
        self.inbox.take()
    }
}
