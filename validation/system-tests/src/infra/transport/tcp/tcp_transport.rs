// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport::Transport;
use faction_protocol::transport_message::TransportMessage;
use std::collections::{HashMap, VecDeque};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn};
use std::time::Duration;

use crate::infra::network::address_book::AddressBook;
use crate::infra::transport::tcp::connection::Connection;
use crate::infra::transport::wire_codec::WireCodec;
use crate::traits::pause::Pause;

pub struct TcpTransport {
    peer_id: PeerId,
    address_book: AddressBook,
    outbound: HashMap<PeerId, Connection>,
    inbound: Arc<Mutex<Vec<Connection>>>,
    received: VecDeque<TransportMessage>,
    pause: Arc<dyn Pause>,
    _listener: TcpListener,
    _shutdown: Arc<AtomicBool>,
    _accept_thread: Option<JoinHandle<()>>,
}

impl Drop for TcpTransport {
    fn drop(&mut self) {
        self._shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self._accept_thread.take() {
            // Wake up accept() by connecting to our own listener
            if let Ok(addr) = self._listener.local_addr() {
                let _ = TcpStream::connect(addr);
            }
            let _ = handle.join();
        }
    }
}

impl TcpTransport {
    pub fn new_mesh(peer_ids: &[PeerId], pause: Arc<dyn Pause>) -> Vec<TcpTransport> {
        let address_book: AddressBook = Arc::new(Mutex::new(Vec::new()));
        peer_ids
            .iter()
            .map(|&peer_id| Self::bind(peer_id, address_book.clone(), pause.clone()))
            .collect()
    }

    fn bind(peer_id: PeerId, address_book: AddressBook, pause: Arc<dyn Pause>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        address_book.lock().unwrap().push((peer_id, addr));
        Self::from_listener(peer_id, listener, address_book, pause)
    }

    #[must_use]
    pub fn join_mesh(
        peer_id: PeerId,
        address_book: AddressBook,
        pause: Arc<dyn Pause>,
    ) -> TcpTransport {
        Self::bind(peer_id, address_book, pause)
    }

    fn from_listener(
        peer_id: PeerId,
        listener: TcpListener,
        address_book: AddressBook,
        pause: Arc<dyn Pause>,
    ) -> Self {
        let inbound = Arc::new(Mutex::new(Vec::new()));
        let shutdown = Arc::new(AtomicBool::new(false));
        let thread_listener = listener.try_clone().unwrap();
        let ib = inbound.clone();
        let sd = shutdown.clone();
        let thread_pause = pause.clone();
        let accept_thread = spawn(move || {
            for incoming in thread_listener.incoming() {
                if sd.load(Ordering::Relaxed) {
                    break;
                }
                match incoming {
                    Ok(stream) => {
                        stream.set_nonblocking(true).unwrap();
                        ib.lock().unwrap().push(Connection::new(stream));
                    }
                    Err(_) => {
                        thread_pause.pause(Duration::from_millis(1));
                    }
                }
            }
        });

        Self {
            peer_id,
            address_book,
            outbound: HashMap::new(),
            inbound,
            received: VecDeque::new(),
            pause,
            _listener: listener,
            _shutdown: shutdown,
            _accept_thread: Some(accept_thread),
        }
    }

    pub fn new(
        listen_addr: SocketAddr,
        peer_id: PeerId,
        peer_addrs: &[(PeerId, SocketAddr)],
        pause: Arc<dyn Pause>,
    ) -> Self {
        let listener = TcpListener::bind(listen_addr).unwrap();

        {
            let probe = TcpStream::connect(listen_addr).unwrap();
            let (conn, _) = listener.accept().unwrap();
            drop(probe);
            drop(conn);
        }

        let address_book: AddressBook = Arc::new(Mutex::new(peer_addrs.to_vec()));
        let mut transport = Self::from_listener(peer_id, listener, address_book, pause.clone());

        for &(pid, addr) in peer_addrs {
            if pid != peer_id {
                let mut attempts = 0;
                let stream = loop {
                    match TcpStream::connect(addr) {
                        Ok(s) => break s,
                        Err(_) if attempts < 150 => {
                            attempts += 1;
                            pause.pause(Duration::from_millis(100));
                        }
                        Err(e) => panic!("failed to connect to {addr}: {e}"),
                    }
                };
                stream.set_nonblocking(true).unwrap();
                transport.outbound.insert(pid, Connection::new(stream));
            }
        }

        transport
    }

    #[must_use]
    pub fn registry(&self) -> AddressBook {
        self.address_book.clone()
    }

    fn connect_to(&mut self, to: PeerId) {
        if self.outbound.contains_key(&to) {
            return;
        }
        let addr = self
            .address_book
            .lock()
            .unwrap()
            .iter()
            .find(|(pid, _)| *pid == to)
            .map(|(_, addr)| *addr);
        if let Some(addr) = addr {
            if let Ok(stream) = TcpStream::connect(addr) {
                stream.set_nonblocking(true).unwrap();
                self.outbound.insert(to, Connection::new(stream));
            }
        }
    }

    fn write_to(&mut self, to: PeerId, data: &[u8]) -> bool {
        let pause = self.pause.clone();
        self.outbound
            .get_mut(&to)
            .is_some_and(|connection| connection.write(data, pause.as_ref()))
    }
}

impl Transport for TcpTransport {
    fn send(&mut self, to: PeerId, message: TransportMessage) {
        if to == self.peer_id {
            return;
        }
        let data = WireCodec::encode(&message);
        for _ in 0..2 {
            self.connect_to(to);
            if !self.outbound.contains_key(&to) {
                return;
            }
            if self.write_to(to, &data) {
                return;
            }
            self.outbound.remove(&to);
        }
    }

    // Every connection read, each one's whole messages queued in the order they
    // arrived: no connection waits behind another's backlog.
    fn recv(&mut self) -> Option<TransportMessage> {
        let mut inbound = self.inbound.lock().unwrap();
        for connection in self.outbound.values_mut().chain(inbound.iter_mut()) {
            connection.fill();
            self.received.extend(connection.messages());
        }
        self.received.pop_front()
    }
}
