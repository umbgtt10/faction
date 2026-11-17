// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::constants::DEFAULT_SUSPICION_THRESHOLD;
use crate::constants::HELPERS;
use crate::infra::fault_settings::FaultSettings;
use crate::infra::node_faults::NodeFaults;
use faction::instance::instance_id::InstanceId;
use faction::quorum::Quorum;
use std::mem::replace;
use std::mem::take;

use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::types::Epoch;
use faction::types::PeerId;

use faction_protocol::commit::Commit;
use faction_protocol::input_message::InputMessage;
use faction_protocol::newcomer::Newcomer;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::protocol::Protocol;
use faction_protocol::timer::Timer;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport::Transport;
use faction_protocol::transport_message::TransportMessage;

use crate::infra::proposal::Proposal;
use crate::membership::Membership;
use crate::node_observer::NodeObserver;
use crate::node_snapshot::NodeSnapshot;
use crate::role::Role;
use crate::traits::roles::Roles;

// A node's loop: its timer and its transport polled in turn, every decision
// carried out. A newcomer waits for its membership and becomes a running node
// of it; a node broadcasts to the members its view shows, read again when an
// admission, a commit or a membership changes them; what it proposes it holds
// until the layer above takes it. A removed node a
// membership invites back becomes a newcomer again, and a restarted node is
// created again at a membership; its roles made anew by `roles`.
pub struct FactionNode {
    instance: InstanceId,
    peers: Vec<PeerId>,
    role: Role,
    transport: Box<dyn Transport>,
    timer: Box<dyn Timer>,
    observer: Box<dyn NodeObserver>,
    faults: NodeFaults,
    roles: Box<dyn Roles>,
    started: bool,
    toggle_timer_and_transport: bool,
    proposals: Vec<Proposal>,
}

impl FactionNode {
    #[must_use]
    pub fn new(
        instance: InstanceId,
        role: Role,
        transport: Box<dyn Transport>,
        timer: Box<dyn Timer>,
        observer: Box<dyn NodeObserver>,
        faults: NodeFaults,
        roles: Box<dyn Roles>,
    ) -> Self {
        let mut node = Self {
            instance,
            peers: Vec::new(),
            role,
            transport,
            timer,
            observer,
            faults,
            roles,
            started: false,
            toggle_timer_and_transport: true,
            proposals: Vec::new(),
        };
        node.read_members();
        node
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

    pub fn change_required_count(&mut self, required_count: usize, epoch: Epoch) {
        self.command(|protocol| {
            protocol.commit(Commit::RequiredCount {
                required_count,
                epoch,
            })
        });
    }

    pub fn commit_addition(&mut self, peer_id: PeerId, epoch: Epoch) {
        self.command(|protocol| protocol.commit(Commit::Addition { peer_id, epoch }));
        self.read_members();
    }

    pub fn commit_removal(&mut self, peer_id: PeerId, epoch: Epoch) {
        self.command(|protocol| protocol.commit(Commit::Removal { peer_id, epoch }));
        self.read_members();
    }

    // The link to `peer_id` cut, as the transport this node's cuts are shared
    // with drops what crosses it.
    pub fn cut(&mut self, peer_id: PeerId) {
        self.faults.cuts.cut(peer_id);
    }

    pub fn decline(&mut self, peer_id: PeerId, epoch: Epoch) {
        self.command(|protocol| protocol.decline(peer_id, epoch));
    }

    pub fn deny(&mut self, peer_id: PeerId) {
        self.command(|protocol| protocol.deny(peer_id));
    }

    pub fn expire_deadline(&mut self) {
        self.command(Protocol::expire_deadline);
    }

    // The instance the node is now: its node, and the incarnation it was last
    // created as.
    #[must_use]
    pub const fn instance(&self) -> InstanceId {
        self.instance
    }

    pub fn member_count(&mut self) -> usize {
        self.snapshot().member_count
    }

    pub fn mend(&mut self, peer_id: PeerId) {
        self.faults.cuts.mend(peer_id);
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

    // The node created again at `membership`, its state gone but its link: a
    // started node opens again at once, one not yet started when it starts.
    pub fn restart(&mut self, membership: &Membership) {
        self.instance = self.next_instance();
        self.role = self.roles.running(self.instance, membership);
        self.read_members();
        if self.started {
            let decisions = match &mut self.role {
                Role::Running(protocol) => Self::opening(protocol.initialize()),
                Role::Waiting(_) => Vec::new(),
            };
            for decision in decisions {
                self.dispatch(decision);
            }
        }
    }

    // The node's transport, timer and clock set to `settings`, as the layers
    // it was built on read them.
    pub fn set_faults(&mut self, settings: &FaultSettings) {
        self.faults.transport.set(settings.transport.clone());
        self.faults.timer.set(settings.timer);
        self.faults.clock.set(settings.clock);
    }

    #[must_use]
    pub fn snapshot(&mut self) -> NodeSnapshot {
        match &mut self.role {
            Role::Running(protocol) => {
                let view = protocol.cluster_view();
                NodeSnapshot {
                    peer_state: view.peer_state(),
                    member_count: view.members().as_slice().len(),
                    members: view.members().as_slice().to_vec(),
                    required_count: view.required_count(),
                    reconfigurations: view.reconfigurations().to_vec(),
                    suspected: view.suspected().to_vec(),
                    quorum: view.quorum(),
                    epoch: view.epoch(),
                }
            }
            Role::Waiting(_) => NodeSnapshot {
                peer_state: PeerState::Fresh,
                member_count: 0,
                members: Vec::new(),
                required_count: 0,
                reconfigurations: Vec::new(),
                suspected: Vec::new(),
                quorum: Quorum::NotTracked,
                epoch: 0,
            },
        }
    }

    pub fn start(&mut self) {
        self.started = true;
        self.observer.on_start();
        let decisions = match &mut self.role {
            Role::Running(protocol) => Self::opening(protocol.initialize()),
            Role::Waiting(_) => Vec::new(),
        };
        for decision in decisions {
            self.dispatch(decision);
        }
    }

    pub fn step(&mut self) -> bool {
        let Some(message) = self.next_input() else {
            self.observer.on_idle();
            return false;
        };
        let decisions = self.decide(message.clone());
        self.observer.on_step(&message, &decisions);
        for decision in decisions {
            self.dispatch(decision);
        }
        true
    }

    // What the node proposed since it was last asked, in the order it
    // proposed it.
    pub fn take_proposals(&mut self) -> Vec<Proposal> {
        take(&mut self.proposals)
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

    // A message decided: a removed node a membership invites back made a
    // newcomer that joins from it, and the members read again after any
    // membership a running node takes.
    fn decide(&mut self, message: InputMessage) -> Vec<OutputMessage> {
        if let (Role::Running(protocol), InputMessage::Transport(transport)) =
            (&mut self.role, &message)
            && protocol.invites(transport)
        {
            self.instance = self.next_instance();
            self.role = self.roles.waiting(self.instance);
        }
        let membership = matches!(
            message,
            InputMessage::Transport(TransportMessage::Membership { .. })
        );
        let decisions = match (&mut self.role, message) {
            (Role::Running(protocol), message) => protocol.decide(message),
            (Role::Waiting(_), InputMessage::Transport(message)) => self.join(message),
            (Role::Waiting(_), InputMessage::Timer(_)) => Vec::new(),
        };
        if membership {
            self.read_members();
        }
        decisions
    }

    fn dispatch(&mut self, decision: OutputMessage) {
        match decision {
            OutputMessage::BroadcastPing { epoch } => {
                for to in &self.peers {
                    if *to != self.instance.node {
                        self.transport.send(
                            *to,
                            TransportMessage::Ping {
                                from: self.instance.node,
                                epoch,
                            },
                        );
                    }
                }
            }
            OutputMessage::BroadcastReady { epoch } => {
                for to in &self.peers {
                    if *to != self.instance.node {
                        self.transport.send(
                            *to,
                            TransportMessage::Ready {
                                from: self.instance.node,
                                epoch,
                            },
                        );
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
            OutputMessage::ProposeAddition { peer_id, epoch } => {
                self.proposals.push(Proposal::Addition { peer_id, epoch });
            }
            OutputMessage::ProposeRemoval { peer_id, epoch } => {
                self.proposals.push(Proposal::Removal { peer_id, epoch });
            }
            OutputMessage::Noop => {}
        }
    }

    // A waiting newcomer given a message: its membership makes it a running
    // node, which opens as a started node does; anything else it lets pass.
    fn join(&mut self, message: TransportMessage) -> Vec<OutputMessage> {
        let placeholder = Role::Waiting(Newcomer::new(
            self.instance,
            DEFAULT_SUSPICION_THRESHOLD,
            1,
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

    // The next input: from the side whose turn it is, its timer or its
    // transport, or from the other if that one holds none -- a node idle only
    // when neither does.
    fn next_input(&mut self) -> Option<InputMessage> {
        let timer_first = self.toggle_timer_and_transport;
        self.toggle_timer_and_transport = !timer_first;
        if timer_first {
            self.timer_input().or_else(|| self.transport_input())
        } else {
            self.transport_input().or_else(|| self.timer_input())
        }
    }

    // The instance the node is created as again: the next incarnation of its
    // node.
    const fn next_instance(&self) -> InstanceId {
        InstanceId {
            node: self.instance.node,
            incarnation: self.instance.incarnation + 1,
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

    fn timer_input(&mut self) -> Option<InputMessage> {
        self.timer
            .poll()
            .map(|TimerEvent::Fire(message)| InputMessage::Timer(message))
    }

    fn transport_input(&mut self) -> Option<InputMessage> {
        self.transport.recv().map(InputMessage::Transport)
    }
}
