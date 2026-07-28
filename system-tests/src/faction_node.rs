// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use crate::node_observer::NodeObserver;
use faction::peer_state::PeerState;
use faction::types::PeerId;

use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::protocol::Protocol;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;

use faction_protocol::timer_trait::Timer;
use faction_protocol::transport_trait::Transport;

pub enum NodeCommand {
    RequestJoin(PeerId, Sender<()>),
    Admit(PeerId, Sender<()>),
    Deny(PeerId, Sender<()>),
    ExpireDeadline(Sender<()>),
    Shutdown,
}

#[derive(Clone, Copy)]
pub struct NodeSnapshot {
    pub peer_state: PeerState,
    pub member_count: usize,
}

impl NodeSnapshot {
    #[must_use]
    pub fn fresh() -> Self {
        Self {
            peer_state: PeerState::Fresh,
            member_count: 0,
        }
    }
}

pub struct FactionNode {
    peer_id: PeerId,
    peers: Vec<PeerId>,
    protocol: Protocol,
    transport: Box<dyn Transport>,
    timer: Box<dyn Timer>,
    observer: Box<dyn NodeObserver>,
    toggle_timer_and_transport: bool,
    idle_delay: Duration,
}

impl FactionNode {
    pub fn new(
        peer_id: PeerId,
        peers: Vec<PeerId>,
        protocol: Protocol,
        transport: Box<dyn Transport>,
        timer: Box<dyn Timer>,
        observer: Box<dyn NodeObserver>,
        idle_delay: Duration,
    ) -> Self {
        Self {
            peer_id,
            peers,
            protocol,
            transport,
            timer,
            observer,
            toggle_timer_and_transport: true,
            idle_delay,
        }
    }

    pub fn start(&mut self) {
        let decisions = self.protocol.initialize();

        self.observer.on_start();
        for decision in decisions {
            self.dispatch(decision);
        }
        self.timer
            .schedule(TimerEvent::Fire(TimerMessage::DeadlineExpired));
    }

    #[must_use]
    pub fn peer_state(&mut self) -> PeerState {
        self.protocol.cluster_view().peer_state()
    }

    pub fn run_until_shutdown(
        &mut self,
        commands: Receiver<NodeCommand>,
        snapshot: Arc<Mutex<NodeSnapshot>>,
    ) {
        self.start();
        self.publish(&snapshot);
        let deadline = Instant::now() + Duration::from_secs(30);
        'outer: loop {
            loop {
                let ack = match commands.try_recv() {
                    Ok(NodeCommand::RequestJoin(peer_id, ack)) => {
                        self.request_join(peer_id);
                        ack
                    }
                    Ok(NodeCommand::Admit(peer_id, ack)) => {
                        self.admit(peer_id);
                        ack
                    }
                    Ok(NodeCommand::Deny(peer_id, ack)) => {
                        self.deny(peer_id);
                        ack
                    }
                    Ok(NodeCommand::ExpireDeadline(ack)) => {
                        self.expire_deadline();
                        ack
                    }
                    Ok(NodeCommand::Shutdown) | Err(TryRecvError::Disconnected) => break 'outer,
                    Err(TryRecvError::Empty) => break,
                };
                self.publish(&snapshot);
                let _ = ack.send(());
            }

            let had_work = self.step_internal();
            self.publish(&snapshot);
            if Instant::now() >= deadline {
                break;
            }
            if !had_work {
                sleep(self.idle_delay);
            }
        }
        self.publish(&snapshot);
    }

    fn publish(&mut self, snapshot: &Arc<Mutex<NodeSnapshot>>) {
        let view = self.protocol.cluster_view();
        let snap = NodeSnapshot {
            peer_state: view.peer_state(),
            member_count: view.members().len(),
        };
        *snapshot.lock().unwrap() = snap;
    }

    fn step_internal(&mut self) -> bool {
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

        let decisions = self.protocol.decide(message.clone());
        self.observer.on_step(&message, &decisions);
        for decision in decisions {
            self.dispatch(decision);
        }
        self.toggle_timer_and_transport = !self.toggle_timer_and_transport;
        true
    }

    pub fn step(&mut self) -> bool {
        self.step_internal()
    }

    pub fn request_join(&mut self, peer_id: PeerId) {
        let _ = self.protocol.request_join(peer_id);
    }

    pub fn admit(&mut self, peer_id: PeerId) {
        let _ = self.protocol.admit(peer_id);
        if !self.peers.contains(&peer_id) {
            self.peers.push(peer_id);
        }
    }

    pub fn deny(&mut self, peer_id: PeerId) {
        let _ = self.protocol.deny(peer_id);
    }

    pub fn expire_deadline(&mut self) {
        let _ = self.protocol.expire_deadline();
    }

    pub fn member_count(&mut self) -> usize {
        self.protocol.cluster_view().members().len()
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
            OutputMessage::Noop => {}
        }
    }
}
