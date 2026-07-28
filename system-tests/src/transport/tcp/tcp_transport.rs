// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::Duration;

pub type AddressBook = Arc<Mutex<Vec<(PeerId, SocketAddr)>>>;

pub struct TcpTransport {
    peer_id: PeerId,
    address_book: AddressBook,
    outbound: HashMap<PeerId, TcpStream>,
    inbound: Arc<Mutex<Vec<TcpStream>>>,
    buf: Vec<u8>,
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
    pub fn new(
        listen_addr: SocketAddr,
        peer_id: PeerId,
        peer_addrs: &[(PeerId, SocketAddr)],
    ) -> Self {
        let listener = TcpListener::bind(listen_addr).unwrap();

        {
            let probe = TcpStream::connect(listen_addr).unwrap();
            let (conn, _) = listener.accept().unwrap();
            drop(probe);
            drop(conn);
        }

        let address_book: AddressBook = Arc::new(Mutex::new(peer_addrs.to_vec()));
        let mut transport = Self::from_listener(peer_id, listener, address_book);

        for &(pid, addr) in peer_addrs {
            if pid != peer_id {
                let mut attempts = 0;
                let stream = loop {
                    match TcpStream::connect(addr) {
                        Ok(s) => break s,
                        Err(_) if attempts < 150 => {
                            attempts += 1;
                            sleep(Duration::from_millis(100));
                        }
                        Err(e) => panic!("failed to connect to {addr}: {e}"),
                    }
                };
                stream.set_nonblocking(true).unwrap();
                transport.outbound.insert(pid, stream);
            }
        }

        transport
    }

    pub fn new_mesh(peer_ids: &[PeerId]) -> Vec<TcpTransport> {
        let address_book: AddressBook = Arc::new(Mutex::new(Vec::new()));
        peer_ids
            .iter()
            .map(|&peer_id| Self::bind(peer_id, address_book.clone()))
            .collect()
    }

    #[must_use]
    pub fn registry(&self) -> AddressBook {
        self.address_book.clone()
    }

    #[must_use]
    pub fn join_mesh(peer_id: PeerId, address_book: AddressBook) -> TcpTransport {
        Self::bind(peer_id, address_book)
    }

    fn bind(peer_id: PeerId, address_book: AddressBook) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        address_book.lock().unwrap().push((peer_id, addr));
        Self::from_listener(peer_id, listener, address_book)
    }

    fn from_listener(peer_id: PeerId, listener: TcpListener, address_book: AddressBook) -> Self {
        let inbound = Arc::new(Mutex::new(Vec::new()));
        let shutdown = Arc::new(AtomicBool::new(false));
        let thread_listener = listener.try_clone().unwrap();
        let ib = inbound.clone();
        let sd = shutdown.clone();
        let accept_thread = spawn(move || {
            for incoming in thread_listener.incoming() {
                if sd.load(Ordering::Relaxed) {
                    break;
                }
                match incoming {
                    Ok(stream) => {
                        stream.set_nonblocking(true).unwrap();
                        ib.lock().unwrap().push(stream);
                    }
                    Err(_) => {
                        sleep(Duration::from_millis(1));
                    }
                }
            }
        });

        Self {
            peer_id,
            address_book,
            outbound: HashMap::new(),
            inbound,
            buf: Vec::new(),
            _listener: listener,
            _shutdown: shutdown,
            _accept_thread: Some(accept_thread),
        }
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
                self.outbound.insert(to, stream);
            }
        }
    }

    fn encode(from: PeerId, tag: u8) -> Vec<u8> {
        let mut data = Vec::with_capacity(9);
        data.extend(&from.to_le_bytes());
        data.push(tag);
        data
    }

    fn decode(data: &[u8]) -> Option<TransportMessage> {
        if data.len() < 9 {
            return None;
        }
        let from = PeerId::from_le_bytes(data[0..8].try_into().ok()?);
        match data[8] {
            0 => Some(TransportMessage::Ping { from }),
            1 => Some(TransportMessage::Ready { from }),
            2 => Some(TransportMessage::Bootstrapped { from }),
            _ => None,
        }
    }

    fn read_streams(streams: &mut dyn Iterator<Item = &mut TcpStream>, buf: &mut Vec<u8>) {
        for stream in streams {
            let mut tmp = [0u8; 64];
            match stream.read(&mut tmp) {
                Ok(0) => continue,
                Ok(n) => buf.extend_from_slice(&tmp[..n]),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
                Err(_) => continue,
            }
        }
    }

    fn write_all(stream: &mut TcpStream, data: &[u8]) -> bool {
        let mut written = 0;
        while written < data.len() {
            match stream.write(&data[written..]) {
                Ok(0) => return false,
                Ok(n) => written += n,
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    sleep(Duration::from_millis(1));
                }
                Err(_) => return false,
            }
        }
        true
    }
}

impl Transport for TcpTransport {
    fn send(&mut self, to: PeerId, message: TransportMessage) {
        if to == self.peer_id {
            return;
        }
        let data = match &message {
            TransportMessage::Ping { from } => Self::encode(*from, 0),
            TransportMessage::Ready { from } => Self::encode(*from, 1),
            TransportMessage::Bootstrapped { from } => Self::encode(*from, 2),
        };
        for _ in 0..2 {
            self.connect_to(to);
            let Some(stream) = self.outbound.get_mut(&to) else {
                return;
            };
            if Self::write_all(stream, &data) {
                return;
            }
            self.outbound.remove(&to);
        }
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        Self::read_streams(&mut self.outbound.values_mut(), &mut self.buf);
        Self::read_streams(&mut self.inbound.lock().unwrap().iter_mut(), &mut self.buf);

        if self.buf.len() >= 9 {
            let msg = Self::decode(&self.buf[..9]);
            self.buf.drain(..9);
            msg
        } else {
            None
        }
    }
}
