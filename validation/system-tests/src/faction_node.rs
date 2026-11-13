// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::constants::DEFAULT_SUSPICION_THRESHOLD;
use crate::constants::HELPERS;
use crate::infra::transport::faulty::link_cuts::LinkCuts;
use faction::quorum::Quorum;
use std::mem::replace;

use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::reconfiguration::Reconfiguration;
use faction::types::PeerId;

use faction_protocol::input_message::InputMessage;
use faction_protocol::newcomer::Newcomer;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::protocol::Protocol;
use faction_protocol::timer::Timer;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport::Transport;
use faction_protocol::transport_message::TransportMessage;

use crate::node_observer::NodeObserver;
use crate::node_snapshot::NodeSnapshot;
use crate::role::Role;

// A node's loop: its timer and its transport polled in turn, every decision
// carried out. A newcomer waits for its membership and becomes a running node
// of it; a node broadcasts to the members its view shows, read again when an
// admission or a commit changes them.
pub struct FactionNode {
    peer_id: PeerId,
    peers: Vec<PeerId>,
    role: Role,
    transport: Box<dyn Transport>,
    timer: Box<dyn Timer>,
    observer: Box<dyn NodeObserver>,
    cuts: LinkCuts,
    toggle_timer_and_transport: bool,
}

impl FactionNode {
    #[must_use]
    pub fn new(
        peer_id: PeerId,
        peers: Vec<PeerId>,
        role: Role,
        transport: Box<dyn Transport>,
        timer: Box<dyn Timer>,
        observer: Box<dyn NodeObserver>,
        cuts: LinkCuts,
    ) -> Self {
        Self {
            peer_id,
            peers,
            role,
            transport,
            timer,
            observer,
            cuts,
            toggle_timer_and_transport: true,
        }
    }

    pub fn abort_addition(&mut self, peer_id: PeerId) {
        self.command(|protocol| protocol.abort_addition(peer_id));
    }

    pub fn abort_removal(&mut self, peer_id: PeerId) {
        self.command(|protocol| protocol.abort_removal(peer_id));
    }

    pub fn admit(&mut self, peer_id: PeerId) {
        self.command(|protocol| protocol.admit(peer_id));
        self.read_members();
    }

    pub fn change_required_count(&mut self, required_count: usize) {
        self.command(|protocol| protocol.change_required_count(required_count));
    }

    pub fn commit_addition(&mut self, peer_id: PeerId) {
        self.command(|protocol| protocol.commit_addition(peer_id));
        self.read_members();
    }

    pub fn commit_removal(&mut self, peer_id: PeerId) {
        self.command(|protocol| protocol.commit_removal(peer_id));
        self.read_members();
    }

    // The link to `peer_id` cut, as the transport this node's cuts are shared
    // with drops what crosses it.
    pub fn cut(&mut self, peer_id: PeerId) {
        self.cuts.cut(peer_id);
    }

    pub fn deny(&mut self, peer_id: PeerId) {
        self.command(|protocol| protocol.deny(peer_id));
    }

    pub fn expire_deadline(&mut self) {
        self.command(Protocol::expire_deadline);
    }

    pub fn member_count(&mut self) -> usize {
        self.snapshot().member_count
    }

    pub fn mend(&mut self, peer_id: PeerId) {
        self.cuts.mend(peer_id);
    }

    #[must_use]
    pub fn peer_state(&mut self) -> PeerState {
        self.snapshot().peer_state
    }

    pub fn request_addition(&mut self, peer_id: PeerId) {
        self.command(|protocol| protocol.request_addition(peer_id));
    }

    pub fn request_join(&mut self, peer_id: PeerId) {
        self.command(|protocol| protocol.request_join(peer_id));
    }

    pub fn request_removal(&mut self, peer_id: PeerId) {
        self.command(|protocol| protocol.request_removal(peer_id));
    }

    #[must_use]
    pub fn snapshot(&mut self) -> NodeSnapshot {
        match &mut self.role {
            Role::Running(protocol) => {
                let view = protocol.cluster_view();
                NodeSnapshot {
                    peer_state: view.peer_state(),
                    member_count: view.members().as_slice().len(),
                    required_count: view.required_count(),
                    reconfiguration: view.reconfiguration(),
                    suspected: view.suspected().to_vec(),
                    quorum: view.quorum(),
                }
            }
            Role::Waiting(_) => NodeSnapshot {
                peer_state: PeerState::Fresh,
                member_count: 0,
                required_count: 0,
                reconfiguration: Reconfiguration::None,
                suspected: Vec::new(),
                quorum: Quorum::NotTracked,
            },
        }
    }

    pub fn start(&mut self) {
        self.observer.on_start();
        let decisions = match &self.role {
            Role::Running(protocol) => Self::opening(protocol.initialize()),
            Role::Waiting(_) => Vec::new(),
        };
        for decision in decisions {
            self.dispatch(decision);
        }
    }

    pub fn step(&mut self) -> bool {
        let message = if self.toggle_timer_and_transport {
            match self.timer.poll() {
                Some(TimerEvent::Fire(timer_msg)) => InputMessage::Timer(timer_msg),
                None => {
                    self.toggle_timer_and_transport = !self.toggle_timer_and_transport;
                    self.observer.on_idle();
                    return false;
                }
            }
        } else {
            match self.transport.recv() {
                Some(transport_msg) => InputMessage::Transport(transport_msg),
                None => {
                    self.toggle_timer_and_transport = !self.toggle_timer_and_transport;
                    self.observer.on_idle();
                    return false;
                }
            }
        };

        let decisions = self.decide(message.clone());
        self.observer.on_step(&message, &decisions);
        for decision in decisions {
            self.dispatch(decision);
        }

        self.toggle_timer_and_transport = !self.toggle_timer_and_transport;
        true
    }

    fn command(&mut self, apply: impl FnOnce(&mut Protocol) -> Vec<OutputMessage>) {
        let decisions = match &mut self.role {
            Role::Running(protocol) => apply(protocol),
            Role::Waiting(_) => Vec::new(),
        };
        for decision in decisions {
            self.dispatch(decision);
        }
    }

    fn decide(&mut self, message: InputMessage) -> Vec<OutputMessage> {
        match (&mut self.role, message) {
            (Role::Running(protocol), message) => protocol.decide(message),
            (Role::Waiting(_), InputMessage::Transport(message)) => self.join(message),
            (Role::Waiting(_), InputMessage::Timer(_)) => Vec::new(),
        }
    }

    fn dispatch(&mut self, decision: OutputMessage) {
        match decision {
            OutputMessage::BroadcastPing => {
                for to in &self.peers {
                    if *to != self.peer_id {
                        self.transport
                            .send(*to, TransportMessage::Ping { from: self.peer_id });
                    }
                }
            }
            OutputMessage::BroadcastReady => {
                for to in &self.peers {
                    if *to != self.peer_id {
                        self.transport
                            .send(*to, TransportMessage::Ready { from: self.peer_id });
                    }
                }
            }
            OutputMessage::Schedule(event) => {
                self.timer.schedule(event);
            }
            OutputMessage::Cancel(event) => {
                self.timer.cancel(event);
            }
            OutputMessage::Send { to, message } => {
                self.transport.send(to, message);
            }
            OutputMessage::ProposeAddition { .. }
            | OutputMessage::ProposeRemoval { .. }
            | OutputMessage::Noop => {}
        }
    }

    // A waiting newcomer given a message: its membership makes it a running
    // node, which opens as a started node does; anything else it lets pass.
    fn join(&mut self, message: TransportMessage) -> Vec<OutputMessage> {
        let placeholder = Role::Waiting(Newcomer::new(
            self.peer_id,
            DEFAULT_SUSPICION_THRESHOLD,
            HELPERS,
            Box::new(NoOpObserver),
        ));
        match replace(&mut self.role, placeholder) {
            Role::Waiting(newcomer) => match newcomer.join(message) {
                Ok((protocol, decisions)) => {
                    self.role = Role::Running(Box::new(protocol));
                    self.read_members();
                    Self::opening(decisions)
                }
                Err(newcomer) => {
                    self.role = Role::Waiting(newcomer);
                    Vec::new()
                }
            },
            running => {
                self.role = running;
                Vec::new()
            }
        }
    }

    // What a node does first, its deadline set with it.
    fn opening(decisions: Vec<OutputMessage>) -> Vec<OutputMessage> {
        decisions
            .into_iter()
            .chain([OutputMessage::Schedule(TimerEvent::Fire(
                TimerMessage::DeadlineExpired,
            ))])
            .collect()
    }

    fn read_members(&mut self) {
        if let Role::Running(protocol) = &mut self.role {
            self.peers = protocol.cluster_view().members().as_slice().to_vec();
        }
    }
}
