// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::quorum_policy::QuorumPolicy;
use faction::reconfiguration::Reconfiguration;
use faction::types::PeerId;

use faction_protocol::input_message::InputMessage;
use faction_protocol::newcomer::Newcomer;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::protocol::Protocol;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;

use crate::in_memory_timer::InMemoryTimer;
use crate::in_memory_transport::InMemoryTransport;
use faction_protocol::timer::Timer;
use faction_protocol::transport::Transport;

// How many of the members a missed probe lists a node asks to probe it.
const HELPERS: usize = 2;

// A deterministic cluster in one process. It keeps no clock for liveness:
// each node's probe period and timeouts are held as the protocol schedules
// them, and fire only when a test says that much time has passed.
pub struct Cluster {
    peer_ids: Vec<PeerId>,
    protocols: Vec<Protocol>,
    timers: Vec<InMemoryTimer>,
    held: Vec<Vec<TimerMessage>>,
    transports: Vec<InMemoryTransport>,
    drops: Vec<DropRule>,
    waiting: Option<WaitingNewcomer>,
    suspicion_threshold: usize,
}

// A newcomer connected to the cluster's transport, waiting for its membership.
struct WaitingNewcomer {
    pub peer_id: PeerId,
    pub newcomer: Newcomer,
    pub transport: InMemoryTransport,
    pub timer: InMemoryTimer,
}

struct DropRule {
    pub from: PeerId,
    pub to: PeerId,
    pub message: TransportMessage,
    pub remaining: usize,
}

impl Cluster {
    pub fn new(count: usize, required: usize, suspicion_threshold: usize) -> Self {
        let peer_ids: Vec<PeerId> = (0..count as PeerId).collect();

        let protocols = peer_ids
            .iter()
            .map(|&id| {
                let config = Config::new(
                    id,
                    peer_ids.clone(),
                    QuorumPolicy::new(required),
                    suspicion_threshold,
                );
                Protocol::new(
                    Faction::new(config, Box::new(NoOpObserver)),
                    peer_ids.clone(),
                    id,
                    HELPERS,
                )
            })
            .collect();

        let timers = (0..count).map(|_| InMemoryTimer::new()).collect();
        let held = (0..count).map(|_| Vec::new()).collect();
        let transports = InMemoryTransport::new_mesh(&peer_ids);

        Self {
            peer_ids,
            protocols,
            timers,
            held,
            transports,
            drops: Vec::new(),
            waiting: None,
            suspicion_threshold,
        }
    }

    pub fn abort_addition(&mut self, index: usize, peer_id: PeerId) {
        let decisions = self.protocols[index].abort_addition(peer_id);
        self.route_all(decisions, index);
    }

    pub fn abort_removal(&mut self, index: usize, peer_id: PeerId) {
        let decisions = self.protocols[index].abort_removal(peer_id);
        self.route_all(decisions, index);
    }

    // A newcomer connected to every member's transport, waiting for the
    // membership that makes it a node of the cluster.
    pub fn add_newcomer(&mut self, peer_id: PeerId) {
        let mut transport = InMemoryTransport::new_mesh(&[peer_id]).remove(0);
        for (index, member) in self.peer_ids.iter().enumerate() {
            transport.connect(*member, &self.transports[index]);
            self.transports[index].connect(peer_id, &transport);
        }
        self.waiting = Some(WaitingNewcomer {
            peer_id,
            newcomer: Newcomer::new(
                peer_id,
                self.suspicion_threshold,
                HELPERS,
                Box::new(NoOpObserver),
            ),
            transport,
            timer: InMemoryTimer::new(),
        });
    }

    pub fn change_required_count(&mut self, index: usize, required_count: usize) {
        let decisions = self.protocols[index].change_required_count(required_count);
        self.route_all(decisions, index);
    }

    pub fn commit_addition(&mut self, index: usize, peer_id: PeerId) {
        let decisions = self.protocols[index].commit_addition(peer_id);
        self.route_all(decisions, index);
    }

    pub fn commit_removal(&mut self, index: usize, peer_id: PeerId) {
        let decisions = self.protocols[index].commit_removal(peer_id);
        self.route_all(decisions, index);
    }

    pub fn drop_message(
        &mut self,
        from: PeerId,
        to: PeerId,
        message: TransportMessage,
        count: usize,
    ) {
        self.drops.push(DropRule {
            from,
            to,
            message,
            remaining: count,
        });
    }

    // The held liveness timer `message` of node `index` fired, as the time it
    // waits for has passed; false if it is not held -- never scheduled, fired
    // or cancelled.
    pub fn elapse(&mut self, index: usize, message: TimerMessage) -> bool {
        let held = self.held[index].iter().position(|held| *held == message);
        held.is_some_and(|position| {
            self.held[index].remove(position);
            let decisions = self.protocols[index].decide(InputMessage::Timer(message));
            self.route_all(decisions, index);
            true
        })
    }

    pub fn inject_timer(&mut self, index: usize, message: TimerMessage) {
        self.timers[index].schedule(TimerEvent::Fire(message));
    }

    pub fn inject_transport(&mut self, index: usize, message: TransportMessage) {
        self.transports[index].push_inbox(message);
    }

    pub fn is_bootstrapped(&mut self) -> bool {
        self.protocols
            .iter_mut()
            .all(|p| p.cluster_view().peer_state() == PeerState::Bootstrapped)
    }

    pub fn is_held(&self, index: usize, message: &TimerMessage) -> bool {
        self.held[index].contains(message)
    }

    pub fn is_newcomer_waiting(&self) -> bool {
        self.waiting.is_some()
    }

    pub fn is_timed_out(&mut self, index: usize) -> bool {
        self.protocols[index].cluster_view().peer_state() == PeerState::TimedOut
    }

    pub fn last_probed(&mut self, index: usize) -> Option<PeerId> {
        self.protocols[index].cluster_view().last_probed()
    }

    pub fn members(&mut self, index: usize) -> Vec<PeerId> {
        self.protocols[index]
            .cluster_view()
            .members()
            .as_slice()
            .to_vec()
    }

    pub fn node_collecting_peers_len(&mut self, index: usize) -> usize {
        self.protocols[index]
            .cluster_view()
            .collecting_peers()
            .len()
    }

    pub fn node_state(&mut self, index: usize) -> PeerState {
        self.protocols[index].cluster_view().peer_state()
    }

    pub fn quorum(&mut self, index: usize) -> Quorum {
        self.protocols[index].cluster_view().quorum()
    }

    // Every node's timer and transport stepped in turn, then the newcomer's,
    // the given number of rounds.
    pub fn reconfiguration(&mut self, index: usize) -> Reconfiguration {
        self.protocols[index].cluster_view().reconfiguration()
    }

    pub fn request_addition(&mut self, index: usize, peer_id: PeerId) {
        let decisions = self.protocols[index].request_addition(peer_id);
        self.route_all(decisions, index);
    }

    pub fn request_removal(&mut self, index: usize, peer_id: PeerId) {
        let decisions = self.protocols[index].request_removal(peer_id);
        self.route_all(decisions, index);
    }

    pub fn run(&mut self, rounds: usize) {
        for _ in 0..rounds {
            for index in 0..self.peer_ids.len() {
                self.step_timer_node(index);
                self.step_transport_node(index);
            }
            self.step_newcomer();
        }
    }

    pub fn start_all(&mut self) {
        for i in 0..self.peer_ids.len() {
            self.start_node(i);
        }
    }

    pub fn start_node(&mut self, index: usize) {
        for decision in self.protocols[index].initialize() {
            self.route(decision, self.peer_ids[index]);
        }
    }

    pub fn step_timer_node(&mut self, index: usize) -> bool {
        if let Some(event) = self.timers[index].poll() {
            let TimerEvent::Fire(tm) = event;
            for decision in self.protocols[index].decide(InputMessage::Timer(tm)) {
                self.route(decision, self.peer_ids[index]);
            }
            true
        } else {
            false
        }
    }

    pub fn step_transport_node(&mut self, index: usize) -> bool {
        if let Some(msg) = self.transports[index].recv() {
            let source = self.peer_ids[index];
            for decision in self.protocols[index].decide(InputMessage::Transport(msg)) {
                self.route(decision, source);
            }
            true
        } else {
            false
        }
    }

    pub fn suspected(&mut self, index: usize) -> Vec<PeerId> {
        self.protocols[index].cluster_view().suspected().to_vec()
    }

    fn is_liveness(message: &TimerMessage) -> bool {
        matches!(
            message,
            TimerMessage::ProbeDue
                | TimerMessage::ProbeMissed { .. }
                | TimerMessage::IndirectProbesMissed { .. }
        )
    }

    fn route(&mut self, msg: OutputMessage, from: PeerId) {
        let index = self.peer_ids.iter().position(|&p| p == from).unwrap();

        match msg {
            OutputMessage::BroadcastPing => {
                let peers = self.peer_ids.clone();
                for &to in &peers {
                    if to != from {
                        let transport_msg = TransportMessage::Ping { from };
                        if self.should_drop(from, to, &transport_msg) {
                            continue;
                        }
                        self.transports[index].send(to, transport_msg);
                    }
                }
            }
            OutputMessage::BroadcastReady => {
                let peers = self.peer_ids.clone();
                for &to in &peers {
                    if to != from {
                        let transport_msg = TransportMessage::Ready { from };
                        if self.should_drop(from, to, &transport_msg) {
                            continue;
                        }
                        self.transports[index].send(to, transport_msg);
                    }
                }
            }
            OutputMessage::Schedule(TimerEvent::Fire(message)) if Self::is_liveness(&message) => {
                self.held[index].push(message);
            }
            OutputMessage::Cancel(TimerEvent::Fire(message)) if Self::is_liveness(&message) => {
                self.held[index].retain(|held| *held != message);
            }
            OutputMessage::Schedule(event) => {
                self.timers[index].schedule(event);
            }
            OutputMessage::Cancel(event) => {
                self.timers[index].cancel(event);
            }
            OutputMessage::Send { to, message } => {
                if !self.should_drop(from, to, &message) {
                    self.transports[index].send(to, message);
                }
            }
            OutputMessage::ProposeAddition { .. }
            | OutputMessage::ProposeRemoval { .. }
            | OutputMessage::Noop => {}
        }
    }

    fn route_all(&mut self, decisions: Vec<OutputMessage>, index: usize) {
        let from = self.peer_ids[index];
        for decision in decisions {
            self.route(decision, from);
        }
    }

    fn should_drop(&mut self, from: PeerId, to: PeerId, msg: &TransportMessage) -> bool {
        for rule in &mut self.drops {
            if rule.from == from && rule.to == to && rule.message == *msg && rule.remaining > 0 {
                rule.remaining -= 1;
                return true;
            }
        }
        false
    }

    // The waiting newcomer given what has reached it: its membership makes it
    // a node of the cluster; anything else it lets pass.
    fn step_newcomer(&mut self) -> bool {
        let Some(mut waiting) = self.waiting.take() else {
            return false;
        };
        let Some(message) = waiting.transport.recv() else {
            self.waiting = Some(waiting);
            return false;
        };
        match waiting.newcomer.join(message) {
            Ok((protocol, decisions)) => {
                self.peer_ids.push(waiting.peer_id);
                self.protocols.push(protocol);
                self.timers.push(waiting.timer);
                self.held.push(Vec::new());
                self.transports.push(waiting.transport);
                self.route_all(decisions, self.peer_ids.len() - 1);
            }
            Err(newcomer) => {
                self.waiting = Some(WaitingNewcomer {
                    newcomer,
                    ..waiting
                });
            }
        }
        true
    }
}
